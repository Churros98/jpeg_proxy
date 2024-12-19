use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};
use tokio::sync::watch::Sender;
use axum::{
    body::{Body, Bytes}, extract::{Path, State}, response::Response, routing::get, Router
};
use tokio::sync::Mutex;
use tokio_stream::wrappers::WatchStream;
use uuid::Uuid;

async fn http_get_stream(
    Path(uuid): Path<String>,
    State(jpeg_watcher): State<Arc<Mutex<HashMap<Uuid, Sender<Result<Bytes, Arc<dyn Error + Sync + Send>>>>>>>
) -> Response<Body> {
    println!("[HTTP] New client connected");

    let uuid = match Uuid::parse_str(&uuid) {
        Ok(uuid) => uuid,
        Err(_) => {
            println!("[HTTP] Invalid UUID: {}", uuid);
            return Response::builder().status(400).body(Body::empty()).unwrap();
        }
    };

    let rx = {
        let jpeg_watcher = jpeg_watcher.lock().await;
        let tx =jpeg_watcher.get(&uuid);
        match tx {
            Some(tx) => tx.subscribe(),
            None => {
                println!("[HTTP] Streamer {} not found", uuid);
                return Response::builder().status(404).body(Body::empty()).unwrap();
            }
        }
    };

    let stream_rx = WatchStream::new(rx);
    let body = Body::from_stream(stream_rx);

    Response::builder()
    .status(200)
    .header(axum::http::header::CONNECTION, "keep-alive")
    .header(axum::http::header::CONTENT_TYPE, "multipart/x-mixed-replace; boundary=--frame")
    .header(axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
    .body(body).unwrap()
}

pub async fn serve(port: i32, jpeg_watcher: Arc<Mutex<HashMap<Uuid, Sender<Result<Bytes, Arc<dyn Error + Sync + Send>>>>>>) {
    println!("[HTTP] Starting HTTP server on port {}", port);

    // Create route
    let app = Router::new()
    .route("/:uuid", get(http_get_stream).with_state(jpeg_watcher.clone()));

    // Start application
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
