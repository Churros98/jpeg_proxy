use tokio::time::sleep;
use tokio::time::Duration;
use tokio::sync::watch::Receiver;
use tokio::{net::{TcpListener, TcpStream}, io::AsyncWriteExt};
use std::time::SystemTime;
use std::io::Error;
use bincode::config;

pub mod actuator;

pub struct Commande {
    port: i32,
    actuator_rx: Receiver<actuator::ActuatorData>,
}

// Gestion du contrôle de la voiture télécommandée (Proxy => Voiture)
impl Commande {
    /// Création
    pub fn new(port: i32, actuator_rx: Receiver<actuator::ActuatorData>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Commande {
            port: port,
            actuator_rx: actuator_rx,
        })
    }

    /// Gestion du serveur et de la connexion d'un client
    pub async fn update(&mut self) {
        println!("[COMMANDE] Démarrage du serveur Contrôleur sur le port {}", self.port);

        let tl_listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await;

        if tl_listener.is_err() {
            println!("[COMMANDE] Impossible de bind le port {}.", self.port);
        }

        let tl_listener = tl_listener.unwrap();

        loop {
            println!("[COMMANDE] En attente d'un client.");
            let (socket, _) = tl_listener.accept().await.unwrap();
            self.stream(socket).await; // Je gére qu'un seul client à la fois
        }
    }

    /// Permet de gérer les erreurs de connexion.
    async fn error(&self, client: &mut TcpStream, e: Error) {
        let client_addr = client.local_addr().unwrap();

        println!("[COMMANDE][{}] ERREUR: {}.", client_addr.to_string(), e);
        client.shutdown().await.expect("[COMMANDE] Impossible de fermer la connexion.\n");
    }

    /// Gestion du stream d'un client
    async fn stream(&mut self, mut client: TcpStream) {
        let client_addr = client.local_addr().unwrap();
        println!("[COMMANDE][{}] Nouveau client connecté.", client_addr.to_string());
        
        let mut start_time = SystemTime::now();
        let mut fps = 0;
        let config = config::standard();

        // Envoi des données de contrôle
        loop  {
            // Récupére la donnée et l'encode
            let actuator_data = self.actuator_rx.borrow().clone();
            let actuator_buffer: Vec<u8> = bincode::encode_to_vec(&actuator_data, config).unwrap();

            // J'envoi la donnée
            match client.try_write(actuator_buffer.as_slice()) {
                Ok(_n) => { fps = fps + 1; }
                Err(e) => {self.error(&mut client, e).await; break;}
            }

            // Affiche le nombre d'itération par secondes.
            if start_time.elapsed().unwrap().as_millis() > 1000 {
                println!("[COMMANDE][{}] DPS: {}", client_addr.to_string(), fps);
                start_time = SystemTime::now();
                fps = 0;
            }

            // J'attend entre les messages (~ 30 message par secondes)
            sleep(Duration::from_millis(33)).await;
        }

        println!("[COMMANDE][{}] Client déconnecté.", client_addr.to_string());
    }
}