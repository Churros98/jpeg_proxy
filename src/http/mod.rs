use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};
use tokio::sync::watch::Sender;
use axum::{
    body::{Body, Bytes}, extract::{Path, State}, response::Response, routing::get, Router
};
use tokio::sync::Mutex;
use tokio_stream::wrappers::WatchStream;
use uuid::Uuid;

async fn http_get_sshot(
    Path(uuid): Path<String>,
    State(jpeg_watcher): State<Arc<Mutex<HashMap<Uuid, Sender<Result<Bytes, Arc<dyn Error + Sync + Send>>>>>>>
) -> Response<Body> {
    println!("[HTTP] New client connected (SSHOT)");

    let uuid = match Uuid::parse_str(&uuid) {
        Ok(uuid) => uuid,
        Err(_) => {
            println!("[HTTP] Invalid UUID: {}", uuid);
            return Response::builder().status(axum::http::StatusCode::BAD_REQUEST).body(Body::empty()).unwrap();
        }
    };

    let mut rx = {
        let jpeg_watcher = jpeg_watcher.lock().await;
        let tx = jpeg_watcher.get(&uuid);
        match tx {
            Some(tx) => tx.subscribe(),
            None => {
                println!("[HTTP] Streamer {} not found (SSHOT)", uuid);
                return Response::builder().status(axum::http::StatusCode::NOT_FOUND).body(Body::empty()).unwrap();
            }
        }
    };

    let jpeg_data = rx.borrow_and_update().clone();

    if let Ok(jpeg_data) = jpeg_data {
        let body = Body::from(jpeg_data);

        Response::builder()
        .status(200)
        .header(axum::http::header::CONNECTION, "keep-alive")
        .header(axum::http::header::CONTENT_TYPE, "image/jpeg")
        .body(body).unwrap()
    } else {
        Response::builder().status(axum::http::StatusCode::BAD_REQUEST).body(Body::empty()).unwrap()
    }
}

async fn http_get_stream(
    Path(uuid): Path<String>,
    State(jpeg_watcher): State<Arc<Mutex<HashMap<Uuid, Sender<Result<Bytes, Arc<dyn Error + Sync + Send>>>>>>>
) -> Response<Body> {
    let uuid = match Uuid::parse_str(&uuid) {
        Ok(uuid) => uuid,
        Err(_) => {
            println!("[HTTP] Invalid UUID: {}", uuid);
            return Response::builder().status(axum::http::StatusCode::BAD_REQUEST).body(Body::empty()).unwrap();
        }
    };

    let rx = {
        let jpeg_watcher = jpeg_watcher.lock().await;
        let tx =jpeg_watcher.get(&uuid);
        match tx {
            Some(tx) => tx.subscribe(),
            None => {
                println!("[HTTP] Streamer {} not found", uuid);
                return Response::builder().status(axum::http::StatusCode::NOT_FOUND).body(Body::empty()).unwrap();
            }
        }
    };

    println!("[HTTP] New client connected for stream {}", uuid);

    let stream_rx = WatchStream::new(rx);
    let body = Body::from_stream(stream_rx);

    Response::builder()
    .status(axum::http::StatusCode::OK)
    .header(axum::http::header::CONNECTION, "keep-alive")
    .header(axum::http::header::CONTENT_TYPE, "multipart/x-mixed-replace; boundary=--frame")
    .header(axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
    .body(body).unwrap()
}

pub async fn serve(port: i32, jpeg_watcher: Arc<Mutex<HashMap<Uuid, Sender<Result<Bytes, Arc<dyn Error + Sync + Send>>>>>>) {
    println!("[HTTP] Starting HTTP server on port {}", port);

    // Create route
    let app = Router::new()
    .route("/stream/:uuid", get(http_get_stream).with_state(jpeg_watcher.clone()))
    .route("/sshot/:uuid", get(http_get_sshot).with_state(jpeg_watcher.clone()));

    // Start application
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
