use std::{cmp::Ordering, io::Error};
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
            // Je récupére la taille de l'image
            let mut size: [u8; 8] = [0; 8];
            match client.read_exact(&mut size).await {
                Ok(_n) => {}
                Err(e) => {self.error(&mut client, e).await; break;}
            }
    
            let size = u64::from_le_bytes(size) as usize;

            // Si l'image est plus grande que 4MB, c'est qu'il y a un problème (~ 1080P)
            if size > 4000000 {
                println!("[JPEG][{}] Image trop grande ({}).", client_addr.to_string(), size);
                break;
            }
            //println!("[DEBUG] Taille de l'image: {} octets", size);
    
            // Je récupére maintenant l'image
            let mut image = vec![0; size];
            match client.read_exact(&mut image).await {
                Ok(_n) => {}
                Err(e) => {self.error(&mut client, e).await; break;}
            }

            // Je vérifie qu'il s'agit bien d'une image JPEG.
            let jpeg_magic: &[u8] = &[0xff, 0xd8, 0xff, 0xe0];
            if image[0..4].cmp(jpeg_magic) != Ordering::Equal {
                println!("[JPEG][{}] Image invalide.", client_addr.to_string());
                break;
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
        if let Err(_e) = self.jpeg_tx.lock().await.send(no_signal_frame) {
            println!("[JPEG] Impossible d'écrire les données dans le watcher.\n");
        }
    }
}