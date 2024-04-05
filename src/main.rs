use tokio::sync::watch;
use tokio::sync::Mutex;
use std::sync::Arc;
use futures::join;
use std::error::Error;

use crate::telemetrie::actuator;
use crate::telemetrie::sensors;

mod http;
mod jpeg;
mod telemetrie;
mod res;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("[CORE] Telemetrie Proxy");

    // Gestion des canaux de communication inter-tâches
    let (jpeg_tx, _) = watch::channel::<Vec<u8>>(res::NO_SIGNAL.to_vec());
    let (sensors_tx, _) = watch::channel::<sensors::SensorsData>(sensors::Sensors::empty());
    let (actuator_tx, actuator_rx) = watch::channel::<actuator::ActuatorData>(actuator::Actuator::empty());

    let jpeg_tx = Arc::new(Mutex::new(jpeg_tx));
    let sensors_tx = Arc::new(Mutex::new(sensors_tx));
    let actuator_tx = Arc::new(Mutex::new(actuator_tx));

    // Préparation des tâches
    let telemetrie = telemetrie::Telemetrie::new(1111, sensors_tx.clone(), actuator_rx)?;
    let jpeg = jpeg::JPEGServer::new(1337, jpeg_tx.clone(), res::NO_SIGNAL)?;

    let telemetrie_task = tokio::spawn(async move {
        let telemetrie = telemetrie;
        let _ = telemetrie.update().await;
    });

    let serveur_jpeg_task = tokio::spawn(async move {
        let jpeg = jpeg;
        let _ = jpeg.update().await;
    });

    let serveur_http_task = http::serve(8000, jpeg_tx.clone(), sensors_tx.clone(), actuator_tx.clone());

    println!("[CORE] Services démarré.");

    // Attend les 
    let _ = join!(serveur_http_task, serveur_jpeg_task, telemetrie_task);
    
    println!("[CORE] Fin du proxy.");
    Ok(())
}
