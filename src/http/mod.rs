use tokio::sync::{watch::Sender, RwLock};
use rand::{distributions::Alphanumeric, Rng};
use axum::{
    body::{Body, Bytes}, extract::{ws::WebSocketUpgrade, ConnectInfo, State}, response::Response, routing::get, Router
};
use tokio::sync::Mutex;
use std::{error::Error, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, sync::{atomic::AtomicU32, Arc}};
use crate::telemetrie::sensors;
use crate::commande::actuator;
use tokio_stream::wrappers::WatchStream;

mod websocket;

async fn http_get_stream(State(tx): State<Arc<Mutex<Sender<Result<Bytes, Arc<dyn std::error::Error + Sync + Send>>>>>>) -> Response<> {
    println!("[HTTP][JPEG] Nouveau client connecté");

    let stream_rx = WatchStream::new(tx.lock().await.subscribe());
    let body = Body::from_stream(stream_rx);

    Response::builder()
    .status(200)
    .header(axum::http::header::CONNECTION, "keep-alive")
    .header(axum::http::header::CONTENT_TYPE, "multipart/x-mixed-replace; boundary=--frame")
    .header(axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
    .body(body).unwrap()
}


async fn http_get_websocket(
    ws: WebSocketUpgrade,
    State(mut wss): State<websocket::WebsocketState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    wss.socket_addr = addr;
    ws.on_upgrade(|socket| websocket::websocket_handle(socket, wss))
}

pub async fn serve(port: i32, jpeg_tx: Arc<Mutex<Sender<Result<Bytes, Arc<dyn Error + Sync + Send>>>>>, sensors_tx: Arc<Mutex<Sender<sensors::SensorsData>>>, actuator_tx: Arc<Mutex<Sender<actuator::ActuatorData>>>) {
    println!("[HTTP] Démarrage du serveur HTTP sur le port {}", port);

    // Génére une clé unique qui permet de s'authentifier en tant que "Pilote"
    let secret_key: String  = rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(30)
    .map(char::from)
    .collect();

    println!("[HTTP][WEBSOCKET] La clé secrete a été générée.");
    println!("[HTTP][WEBSOCKET] Clé: {}", secret_key);

    // Prépare la structure pour la gestion des WebSocket
    let wss = websocket::WebsocketState {
        sensors_tx: sensors_tx,
        actuator_tx: actuator_tx,
        pilot_id: Arc::new(RwLock::new(AtomicU32::new(0))),
        client_id: 0,
        socket_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 1)),
        secret_key: secret_key,
    };

    // Créer la route
    let app = Router::new()
    .route("/stream", get(http_get_stream).with_state(jpeg_tx.clone()))
    .route("/ws", get(http_get_websocket)).with_state(wss);

    // Démarre l'application
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
