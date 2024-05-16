use std::cmp::Ordering;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::SystemTime;
use rand::Rng;
use serde::Serialize;
use tokio::sync::RwLock;
use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use futures::stream::SplitStream;
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::watch::Sender;
use crate::commande::actuator;
use crate::commande::actuator::Actuator;
use crate::commande::actuator::ActuatorData;
use crate::telemetrie::sensors;
use tokio::time::Duration;
use tokio::time::sleep;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct WebsocketState {
    pub sensors_tx: Arc<Mutex<Sender<sensors::SensorsData>>>,
    pub actuator_tx: Arc<Mutex<Sender<actuator::ActuatorData>>>,
    pub pilot_id: Arc<RwLock<AtomicU32>>,
    pub client_id: u32,
    pub socket_addr: SocketAddr,
    pub secret_key: String,
}

// Une structure qui permet la fusion des données de la voiture et du proxy.
#[derive(Clone, Serialize, Copy)]
struct RCStatus {
    client_id: u32,
    pilot_id: u32,
    message_date: f32,
    sensors: sensors::SensorsData,
}

/// Gestion d'une nouvelle connexion
pub async fn websocket_handle(socket: WebSocket, mut wss: WebsocketState) {
    let (sender, receiver) = socket.split();

    let ip =  wss.socket_addr.ip().to_string();

    // Je me génére un ID en tant que "client" que je stock dans la structure.
    wss.client_id = rand::thread_rng().gen();

    println!("[HTTP][WEBSOCKET] Nouvelle connexion de {} [ID: {}].", ip, wss.client_id);

    // Créer des tâches spécifique pour la réception/émission.
    let mut sender = tokio::spawn(websocket_sender(sender, wss.clone()));
    let mut reader = tokio::spawn(websocket_reader(receiver, wss.clone()));

    // Si une task est fini, alors je ferme la connexion.
    tokio::select! {
        _ = (&mut sender) => {
            reader.abort();
        },
        _ = (&mut reader) => {
            sender.abort();
        }
    }

    println!("[HTTP][WEBSOCKET] Fin de connexion de {} [ID: {}].", ip, wss.client_id);

    // Si je suis le pilote et que la connexion est terminée, j'envoi à la voiture un ordre d'arrêt.
    if wss.pilot_id.read().await.load(core::sync::atomic::Ordering::Relaxed) == wss.client_id {
        let _ = wss.actuator_tx.lock().await.send(Actuator::empty());
    }
}

/// Gestion de l'envoi des données de télémétrie aux clients connectés.
pub async fn websocket_sender(mut sender: SplitSink<WebSocket, Message>, wss: WebsocketState) {
    let mut sensors_rx = wss.sensors_tx.lock().await.subscribe();

    let mut last_message = SystemTime::now();

    loop {
        // Récupére la dernière valeur renvoyée par la voiture
        let sensors_data = *sensors_rx.borrow_and_update();
        
        if sensors_rx.has_changed().unwrap_or(false) {
            last_message = SystemTime::now();
        }

        let pilot_id: u32;
        {
            pilot_id = wss.pilot_id.read().await.load(core::sync::atomic::Ordering::Relaxed);
        }

        // Prépare une structure qui contient l'ensemble des données utiles
        let data = RCStatus {
            client_id: wss.client_id,
            pilot_id: pilot_id,
            message_date: last_message.elapsed().unwrap().as_secs_f32(),
            sensors: sensors_data,
        };

        // Prépare et envoi les données.
        let json = serde_json::to_string(&data).unwrap_or(String::new());
        if sender.send(Message::Text(json)).await.is_err() {
            break;
        }

        sleep(Duration::from_millis(20)).await;
    }
}

/// Gestion de la réception des données de commande des clients connectés.
pub async fn websocket_reader(mut receiver: SplitStream<WebSocket>, wss: WebsocketState) {
    // Permet de recevoir un message
    while let Some(msg) = receiver.next().await {

        // Si le message est valide
        if let Ok(msg) = msg {
            let msg = msg.to_text();

            // Je transforme le message en texte et je vérifie qu'il n'y et pas d'erreur
            if !msg.is_err() {
                let msg = msg.unwrap();

                // Si le message est égal à la clé, alors je deviens pilote.
                if msg.cmp(wss.secret_key.as_str()) == Ordering::Equal {
                    {
                        wss.pilot_id.write().await.store(wss.client_id, core::sync::atomic::Ordering::Relaxed);
                    }
                    println!("[HTTP][WEBSOCKET] Changement de pilote [ID: {}]", wss.client_id);
                    continue;
                }

                // Vérifie si j'ai bien la permission d'envoyer des messages à la voiture.
                if wss.pilot_id.read().await.load(core::sync::atomic::Ordering::Relaxed) == wss.client_id {
                    // Je fait la conversion JSON => Structure
                    let actuator_data: Result<ActuatorData, serde_json::Error> = serde_json::from_str(msg);
                    if actuator_data.is_err() {
                        println!("[HTTP][WEBSOCKET] Donnée invalide du pilote.");
                    } else {
                        let _ = wss.actuator_tx.lock().await.send(actuator_data.unwrap());
                    }
                } else {
                    println!("[HTTP][WEBSOCKET] Message envoyé sans permission [ID: {}]", wss.client_id);
                }
            } else {
                println!("[HTTP][WEBSOCKET] Conversion en texte impossible.");
            }
        }

        sleep(Duration::from_millis(20)).await;
    }
}