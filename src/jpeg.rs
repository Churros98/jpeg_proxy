use std::io::Error;
use std::time::SystemTime;
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::{net::{TcpListener, TcpStream}, sync::watch::Sender, io::{AsyncReadExt, AsyncWriteExt}};


pub struct JPEGServer {
    port: i32,
    jpeg_tx: Arc<Mutex<Sender<Vec<u8>>>>,
    no_signal: Box<[u8]>,
}

impl JPEGServer {
    pub fn new(port: i32, jpeg_tx: Arc<Mutex<Sender<Vec<u8>>>>, no_signal: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(JPEGServer {
            port: port,
            jpeg_tx: jpeg_tx,
            no_signal: Box::from(no_signal),
        })
    }

    pub async fn update(&self) {
        println!("[JPEG] Démarrage du serveur JPEG sur le port {}", self.port);

        let jpeg_listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await;
    
        if jpeg_listener.is_err() {
            println!("[JPEG] Impossible de bind le port {}.", self.port);
        }
    
        let jpeg_listener = jpeg_listener.unwrap();
    
        loop {
            println!("[JPEG] En attente d'un client.");
            let (socket, _) = jpeg_listener.accept().await.unwrap();
            self.stream(socket).await; // Je gére qu'un seul client à la fois
        }
    }

    async fn error(&self, client: &mut TcpStream, e: Error) {
        let client_addr = client.local_addr().unwrap();
    
        println!("[JPEG][{}] ERREUR: {}.", client_addr.to_string(), e);
        client.shutdown().await.expect("[JPEG] Impossible de fermer la connexion.\n");
    }
    
    async fn stream(&self, mut client: TcpStream) {
        let client_addr = client.local_addr().unwrap();
        println!("[JPEG][{}] Nouveau client connecté.", client_addr.to_string());
        
        let mut start_time = SystemTime::now();
        let mut fps = 0;
    
        loop  {
            let mut size: [u8; 8] = [0; 8];
            match client.read_exact(&mut size).await {
                Ok(_n) => {}
                Err(e) => {self.error(&mut client, e).await; break;}
            }
    
            let size = u64::from_le_bytes(size) as usize;
            //println!("[DEBUG] Taille de l'image: {} octets", size);
    
            let mut image = vec![0; size];
            match client.read_exact(&mut image).await {
                Ok(_n) => {}
                Err(e) => {self.error(&mut client, e).await; break;}
            }
    
            let in_stream = self.jpeg_tx.lock().await.send(image).is_ok();

            fps = fps + 1;
        
            if start_time.elapsed().unwrap().as_millis() > 1000 {
                println!("[JPEG][{}] FPS: {} (stream?: {})", client_addr.to_string(), fps, in_stream);
                start_time = SystemTime::now();
                fps = 0;
            }
        }
    
        println!("[JPEG][{}] Client déconnecté.", client_addr.to_string());
        self.jpeg_tx.lock().await.send(self.no_signal.to_vec()).expect("[JPEG] Impossible d'écrire les données dans le watcher.\n");
    }
}