use tokio::sync::{watch::Receiver, Mutex};
use std::sync::Arc;
use tokio::{net::{TcpListener, TcpStream}, sync::watch::Sender, io::{AsyncReadExt, AsyncWriteExt}};
use std::{time::SystemTime, usize};
use std::io::Error;
use bincode::{config, error::DecodeError};

use crate::telemetrie::sensors::{Sensors, SensorsData};

pub mod sensors;
pub mod actuator;

pub struct Telemetrie {
    port: i32,
    sensors_tx: Arc<Mutex<Sender<sensors::SensorsData>>>,
    actuator_rx: Receiver<actuator::ActuatorData>,
}

impl Telemetrie {
    /// Création 
    pub fn new(port: i32, sensors_tx: Arc<Mutex<Sender<sensors::SensorsData>>>, actuator_rx: Receiver<actuator::ActuatorData>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Telemetrie { 
            port: port,
            sensors_tx: sensors_tx,
            actuator_rx: actuator_rx,
        })
    }

    /// Gestion du serveur et de la connexion d'un client
    pub async fn update(&self) {
        println!("[TELEMETRIE] Démarrage du serveur TELEMETRIE sur le port {}", self.port);

        let tl_listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await;

        if tl_listener.is_err() {
            println!("[TELEMETRIE] Impossible de bind le port {}.", self.port);
        }

        let tl_listener = tl_listener.unwrap();

        loop {
            println!("[TELEMETRIE] En attente d'un client.");
            let (socket, _) = tl_listener.accept().await.unwrap();
            self.stream(socket, &self.sensors_tx).await; // Je gére qu'un seul client à la fois
        }
    }

    /// Permet de gérer les erreurs de connexion.
    async fn error(&self, client: &mut TcpStream, e: Error) {
        let client_addr = client.local_addr().unwrap();

        println!("[TELEMETRIE][{}] ERREUR: {}.", client_addr.to_string(), e);
        client.shutdown().await.expect("[TELEMETRIE] Impossible de fermer la connexion.\n");
    }

    /// Gestion du stream d'un client
    async fn stream(&self, mut client: TcpStream, sensors_tx: &Arc<Mutex<Sender<sensors::SensorsData>>>) {
        let client_addr = client.local_addr().unwrap();
        println!("[TELEMETRIE][{}] Nouveau client connecté.", client_addr.to_string());
        
        let mut start_time = SystemTime::now();
        let mut fps = 0;
        let config = config::standard();

        // Réception des données de télémétrie et envoi des commandes
        loop  {
            // Je prépare un buffer avec des données vide à l'intérieur, puis je réceptionne les données de télémétrie
            let mut buf: Vec<u8> = bincode::encode_to_vec(&Sensors::empty(), config).unwrap();
            match client.read_exact(&mut buf).await {
                Ok(_n) => {}
                Err(e) => {self.error(&mut client, e).await; break;}
            }

            // Je décode les données de télémétrie, et je les envois dans le channel.
            let decoder: Result<(SensorsData, usize), DecodeError> = bincode::decode_from_slice(&buf[..], config);
            if decoder.is_err() {
                println!("[TELEMETRIE] ERREUR: Impossible de décoder l'objet !");
            } else {
                let (sensors_data, _len) = decoder.unwrap();
                let _ = sensors_tx.lock().await.send(sensors_data);
                fps = fps + 1;
            }

            // J'envoi en retour des données de commandes
            // Prépare le buffer et écrit les données de télémétrie
            let actuator_data = *self.actuator_rx.borrow();
            let actuator_buffer: Vec<u8> = bincode::encode_to_vec(&actuator_data, config).unwrap();
            match client.try_write(actuator_buffer.as_slice()) {
                Ok(_n) => {}
                Err(e) => {self.error(&mut client, e).await; break;}
            }

            // Affiche le nombre d'itération par secondes.
            if start_time.elapsed().unwrap().as_millis() > 1000 {
                println!("[TELEMETRIE][{}] DPS: {}", client_addr.to_string(), fps);
                start_time = SystemTime::now();
                fps = 0;
            }
        }

        println!("[TELEMETRIE][{}] Client déconnecté.", client_addr.to_string());
    }
}