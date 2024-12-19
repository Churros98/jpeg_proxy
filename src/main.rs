use axum::body::Bytes;
use tokio::signal;
use tokio::sync::watch;
use tokio::sync::Mutex;
use uuid::Uuid;
use std::sync::Arc;
use futures::join;
use std::error::Error;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

#[cfg(unix)]
use tokio::signal::unix::SignalKind;

mod http;
mod jpeg;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token = CancellationToken::new();

    println!("[CORE] JPEG Proxy");

    // Multiple channels management
    let jpeg_watcher = Arc::new(Mutex::new(HashMap::<Uuid, watch::Sender<Result<Bytes, Arc<dyn std::error::Error + Sync + Send>>>>::new()));

    let jpeg_server = jpeg::JPEGServer::new(1337, jpeg_watcher.clone(), token.child_token());

    let serveur_jpeg_task = jpeg_server.run();
    let serveur_http_task = http::serve(8000, jpeg_watcher.clone());

    // Check if all tasks are finished
    let _ = join!(serveur_http_task, serveur_jpeg_task);

    println!("[CORE] Services started.");

    #[cfg(unix)]
    {
        let mut test = tokio::signal::unix::signal(SignalKind::interrupt()).unwrap();
        tokio::select! {
            _ = test.recv() => {
                println!("Interrupt signal received");
                token.cancel();
            },
            _ = signal::ctrl_c() => {
                println!("Ctrl+C signal received");
                token.cancel();
            },
        }
    }

    #[cfg(not(unix))]
    {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("Ctrl+C signal received");
                token.cancel();
            },
        }
    }

    println!("[CORE] Terminated.");
    Ok(())
}
