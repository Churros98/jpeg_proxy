use std::collections::HashMap;
use std::io::Error;
use std::time::SystemTime;
use axum::body::Bytes;
use tokio::sync::{watch, Mutex};
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use uuid::Uuid;
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};

pub struct JPEGClient {
    client: TcpStream,
    uuid: Uuid,
    jpeg_watcher: Arc<Mutex<HashMap<Uuid, watch::Sender<Result<Bytes, Arc<dyn std::error::Error + Sync + Send>>>>>>,
    token: CancellationToken,
}

impl JPEGClient {
    pub fn new(client: TcpStream, uuid: Uuid, jpeg_watcher: Arc<Mutex<HashMap<Uuid, watch::Sender<Result<Bytes, Arc<dyn std::error::Error + Sync + Send>>>>>>, token: CancellationToken) -> Self {
        println!("[JPEG][{}] New client connected with IP [{}].", uuid.to_string(), client.peer_addr().unwrap().to_string());

        JPEGClient {
            client,
            jpeg_watcher,
            uuid,
            token,
        }
    }

    fn prepare_frame(&self, image: Vec<u8>) -> axum::body::Bytes {
        // Prepare data for streaming
        let start_frame = format!("--frame\r\nContent-type: image/jpeg\r\nContent-Lenght: {}\r\n\r\n", image.len()).as_bytes().to_vec();
        let frame = [start_frame, image].concat();
        let frame_body = axum::body::Bytes::from(frame);

        frame_body
    }

    async fn recv_frames(&mut self) {
        let mut start_time = SystemTime::now();
        let mut fps = 0;
        let mut lock_time_total = std::time::Duration::from_secs(0);
        
        while !self.token.is_cancelled() {
            // Get the image size
            let mut size: [u8; 8] = [0; 8];
            match self.client.read_exact(&mut size).await {
                Ok(_n) => {}
                Err(e) => {self.error(e); break;}
            }
    
            let size = u64::from_le_bytes(size) as usize;

            // If image is larger than 4MB, there's a problem (~ 1080P)
            if size > 4000000 {
                println!("[JPEG][{}] Image too large. ({}).", self.uuid.to_string(), size);
                break;
            }
    
            // Now get the image
            let mut image = vec![0; size];
            match self.client.read_exact(&mut image).await {
                Ok(_) => {fps = fps + 1;}
                Err(e) => {self.error(e); break;}
            }

            // Prepare data for streaming and send it to the channel
            let frame = self.prepare_frame(image);
            
            let lock_start = SystemTime::now();
            {
                let jpeg_watcher = self.jpeg_watcher.lock().await;
                let _ = jpeg_watcher.get(&self.uuid).unwrap().send(Ok(frame));
                lock_time_total += lock_start.elapsed().unwrap();
            }

            // Calculate FPS
            if start_time.elapsed().unwrap().as_millis() > 1000 {
                println!("[JPEG][{}] FPS: {}, Lock time: {:?}ms", 
                    self.uuid.to_string(), 
                    fps,
                    lock_time_total.as_millis() as f64);
                start_time = SystemTime::now();
                lock_time_total = std::time::Duration::from_secs(0);
                fps = 0;
            }
        }

        self.stop().await;
    }

    fn error(&mut self, e: Error) {
        println!("[JPEG][{}] ERROR: {}.", self.uuid.to_string(), e);
    }

    async fn stop(&mut self) {
        let _ = self.client.shutdown().await;
        {
            let mut jpeg_watcher = self.jpeg_watcher.lock().await;
            jpeg_watcher.remove(&self.uuid);
        }
        println!("[JPEG][{}] Client disconnected.", self.uuid.to_string());
    }
}


pub struct JPEGServer {
    port: i32,
    jpeg_watcher: Arc<Mutex<HashMap<Uuid, watch::Sender<Result<Bytes, Arc<dyn std::error::Error + Sync + Send>>>>>>,
    token: CancellationToken,
}

impl JPEGServer {
    pub fn new(port: i32, jpeg_watcher: Arc<Mutex<HashMap<Uuid, watch::Sender<Result<Bytes, Arc<dyn std::error::Error + Sync + Send>>>>>>, token: CancellationToken) -> Self {
        JPEGServer {
            port: port,
            jpeg_watcher: jpeg_watcher,
            token: token,
        }
    }

    pub async fn run(&self) {
        println!("[JPEG] Starting JPEG server on port {}", self.port);

        let jpeg_listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await;
    
        if jpeg_listener.is_err() {
            println!("[JPEG] Unable to bind port {}.", self.port);
        }
    
        let jpeg_listener = jpeg_listener.unwrap();

        while !self.token.is_cancelled() {
            println!("[JPEG] Waiting for client.");
            let (mut socket, _) = jpeg_listener.accept().await.unwrap();

            let client_addr = socket.peer_addr().unwrap().to_string();
            println!("[JPEG][{}] New client connected.", client_addr.to_string());

            // Receive client UUID (first message)
            let mut buffer = [0; 36];
            match socket.read_exact(&mut buffer).await {
                Ok(_) => {}
                Err(_) => { println!("[JPEG][{}] Error receiving UUID.", client_addr); let _ = socket.shutdown().await; continue; }
            }

            let uuid = String::from_utf8_lossy(&buffer).to_string();
            let uuid = match Uuid::parse_str(&uuid) {
                Ok(uuid) => uuid,
                Err(_) => {
                    println!("[JPEG][{}] Invalid UUID [{}].", client_addr, uuid);
                    let _ = socket.shutdown().await;
                    continue;
                }
            };

            // Create a channel for the client
            let (tx, _) = watch::channel(Ok(Bytes::new()));
            // Add channel to watcher
            {
                let mut jpeg_watcher = self.jpeg_watcher.lock().await;
                jpeg_watcher.insert(uuid, tx);
            }

            // Create task for client
            let mut client = JPEGClient::new(socket, uuid, self.jpeg_watcher.clone(), self.token.clone());
            tokio::spawn(async move {
                client.recv_frames().await;
            });
        }

        println!("[JPEG] JPEG server stopped.");
    }
}