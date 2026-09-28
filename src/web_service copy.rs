use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, future::BoxFuture};
use tokio::sync::broadcast::{self, Receiver, Sender, error::RecvError};
use tracing::warn;

use crate::notification::Notif;

pub struct NotifStream {
    notif_tx: Sender<Notif>,
    recv_fut: BoxFuture<'static, (Receiver<Notif>, Result<Notif, RecvError>)>,
}

impl NotifStream {
    pub fn new() -> (Self, Sender<Notif>) {
        let (tx, rx) = broadcast::channel(4);

        let this = Self {
            notif_tx: tx.clone(),
            recv_fut: Box::pin(recv_once(rx)),
        };

        (this, tx)
    }
}

async fn recv_once(mut rx: Receiver<Notif>) -> (Receiver<Notif>, Result<Notif, RecvError>) {
    let result = rx.recv().await;
    (rx, result)
}

impl Stream for NotifStream {
    type Item = Notif;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            let (rx, result) = match this.recv_fut.as_mut().poll(cx) {
                Poll::Ready(output) => output,
                Poll::Pending => return Poll::Pending,
            };

            // The completed future's `Receiver` is reused for the next
            // `recv()` call, and a fresh future is kept pinned in `self` so
            // its waker registration survives across polls.
            this.recv_fut = Box::pin(recv_once(rx));

            match result {
                Ok(notif) => return Poll::Ready(Some(notif)),
                Err(RecvError::Closed) => return Poll::Ready(None),
                Err(RecvError::Lagged(missed)) => {
                    warn!(
                        missed,
                        "notification receiver lagged, dropping missed notifications"
                    );
                }
            }
        }
    }
}

impl Clone for NotifStream {
    fn clone(&self) -> Self {
        let rx = self.notif_tx.subscribe();

        Self {
            notif_tx: self.notif_tx.clone(),
            recv_fut: Box::pin(recv_once(rx)),
        }
    }
}
