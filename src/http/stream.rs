use tokio::sync::watch::Receiver;
use futures_core::task::{Context, Poll};
use std::pin::Pin;
use axum::body::Bytes;
use std::error::Error;
use futures_core::stream::Stream;

pub struct JPEGStream {
    pub rx: Receiver<Vec<u8>>,
}

impl Stream for JPEGStream {
    type Item = Result<Bytes, Box<dyn Error + Send + Sync>>;
    
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Récupérer la référence à la structure
        let this: Pin<&mut JPEGStream> = self;

        // Prépare la future frame
        let image = this.rx.borrow().to_vec();

        let start_frame = format!("--frame\r\nContent-type: image/jpeg\r\nContent-Lenght: {}\r\n\r\n", image.len()).as_bytes().to_vec();
        let result = [start_frame, image].concat();
        let frame = axum::body::Bytes::from(result);
        return Poll::Ready(Some(Ok(frame)));
    }
}