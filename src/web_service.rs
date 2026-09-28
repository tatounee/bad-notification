use std::{
    pin::Pin,
    task::{Context, Poll, ready},
};

use futures::Stream;
use pin_project_lite::pin_project;
use tokio::sync::broadcast::{self, Sender};
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};
use tracing::warn;

use crate::notification::Notif;

pin_project! {
    pub struct NotifStream {
        notif_tx: Sender<Notif>,
        #[pin]
        stream: BroadcastStream<Notif>,
    }
}

impl NotifStream {
    pub fn new() -> (Self, Sender<Notif>) {
        let (tx, rx) = broadcast::channel(4);

        let this = Self {
            notif_tx: tx.clone(),
            stream: BroadcastStream::new(rx),
        };

        (this, tx)
    }
}

impl Stream for NotifStream {
    type Item = Notif;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            let notif = ready!(this.stream.as_mut().poll_next(cx));

            let Some(notif) = notif else {
                return Poll::Ready(None);
            };

            match notif {
                Ok(notif) => return Poll::Ready(Some(notif)),
                Err(BroadcastStreamRecvError::Lagged(n)) => {
                    warn!("Miss {n} notification from lagging");
                }
            }
        }
    }
}

impl Clone for NotifStream {
    fn clone(&self) -> Self {
        let stream = BroadcastStream::new(self.notif_tx.subscribe());

        Self {
            notif_tx: self.notif_tx.clone(),
            stream,
        }
    }
}
