use tokio::sync::watch::Receiver;
use futures_core::task::{Context, Poll};
use std::pin::Pin;
use axum::body::Bytes;
use std::error::Error;
use futures_core::stream::Stream;
use std::time::Instant;

pub struct JPEGStream {
    pub last_send: Instant,
    pub rx: Receiver<Vec<u8>>,
}

impl Stream for JPEGStream {
    type Item = Result<Bytes, Box<dyn Error + Send + Sync>>;
    
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Récupérer la référence à la structure
        let mut this: Pin<&mut JPEGStream> = self;

        // Prépare la future frame
        if this.last_send.elapsed().as_secs() > 1 {
            let image = this.rx.borrow_and_update().to_vec();
            println!("Frame envoyée ({} bytes) (time?: {})", image.len(), this.last_send.elapsed().as_secs());

            let start_frame = format!("--frame\r\nContent-type: image/jpeg\r\nContent-Lenght: {}\r\n\r\n", image.len()).as_bytes().to_vec();
            let result = [start_frame, image].concat();
            let frame = axum::body::Bytes::from(result);

            this.last_send = Instant::now();
            return Poll::Ready(Some(Ok(frame)));
        } else {
            return Poll::Pending
        }
    }
}