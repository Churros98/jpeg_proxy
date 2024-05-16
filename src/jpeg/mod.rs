use std::io::Error;
use std::time::SystemTime;
use axum::body::Bytes;
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::{net::{TcpListener, TcpStream}, sync::watch::Sender, io::{AsyncReadExt, AsyncWriteExt}};


pub struct JPEGServer {
    port: i32,
    jpeg_tx: Arc<Mutex<Sender<Result<Bytes, Arc<dyn std::error::Error + Sync + Send>>>>>,
    no_signal: Box<[u8]>,
}

impl JPEGServer {
    pub fn new(port: i32, jpeg_tx: Arc<Mutex<Sender<Result<Bytes, Arc<dyn std::error::Error + Sync + Send>>>>>, no_signal: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
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
    
    fn prepare_frame(&self, image: Vec<u8>) -> Result<axum::body::Bytes, Arc<dyn std::error::Error + Sync + Send>> {
            // Je prépare les données pour le stream
            let start_frame = format!("--frame\r\nContent-type: image/jpeg\r\nContent-Lenght: {}\r\n\r\n", image.len()).as_bytes().to_vec();
            let frame = [start_frame, image].concat();
            let frame_body = axum::body::Bytes::from(frame);

            Ok(frame_body)
    }

    async fn stream(&self, mut client: TcpStream) {
        let client_addr = client.local_addr().unwrap();
        println!("[JPEG][{}] Nouveau client connecté.", client_addr.to_string());
        
        let no_signal_frame = self.prepare_frame(self.no_signal.to_vec());
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

            // Je prépare les données pour le stream
            let frame = self.prepare_frame(image);

            let in_stream = self.jpeg_tx.lock().await.send(frame).is_ok();

            fps = fps + 1;
        
            if start_time.elapsed().unwrap().as_millis() > 1000 {
                println!("[JPEG][{}] FPS: {} (stream?: {})", client_addr.to_string(), fps, in_stream);
                start_time = SystemTime::now();
                fps = 0;
            }
        }
    
        println!("[JPEG][{}] Client déconnecté.", client_addr.to_string());
        self.jpeg_tx.lock().await.send(no_signal_frame).expect("[JPEG] Impossible d'écrire les données dans le watcher.\n");
    }
}