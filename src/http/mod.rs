use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::sync::watch::Sender;
use axum::{
    body::{Body, Bytes}, extract::State, response::Response, routing::get, Router
};
use tokio::sync::Mutex;
use tokio_stream::wrappers::WatchStream;

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

pub async fn serve(port: i32, jpeg_tx: Arc<Mutex<Sender<Result<Bytes, Arc<dyn Error + Sync + Send>>>>>) {
    println!("[HTTP] Démarrage du serveur HTTP sur le port {}", port);

    // Créer la route
    let app = Router::new()
    .route("/stream", get(http_get_stream).with_state(jpeg_tx.clone()));

    // Démarre l'application
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
