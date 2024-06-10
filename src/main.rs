use axum::body::Bytes;
use tokio::sync::watch;
use tokio::sync::Mutex;
use std::sync::Arc;
use futures::join;
use std::error::Error;

mod http;
mod jpeg;
mod res;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("[CORE] Telemetrie Proxy");

    // Gestion des canaux de communication inter-tâches
    let (jpeg_tx, _) = watch::channel::<Result<Bytes, Arc<dyn Error + Sync + Send>>>(Ok(axum::body::Bytes::new()));

    let jpeg_tx = Arc::new(Mutex::new(jpeg_tx));

    // Préparation des tâches
    let jpeg = jpeg::JPEGServer::new(1337, jpeg_tx.clone(), res::NO_SIGNAL)?;

    let serveur_jpeg_task = tokio::spawn(async move {
        let jpeg = jpeg;
        let _ = jpeg.update().await;
    });

    let serveur_http_task = http::serve(8000, jpeg_tx.clone());

    println!("[CORE] Services démarré.");

    // Vérifie que toutes les tâches soit terminées.
    let _ = join!(serveur_http_task, serveur_jpeg_task);
    
    println!("[CORE] Fin du proxy.");
    Ok(())
}
