pub mod orchestrator_protocol;
pub mod runner_protocol;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub async fn spawn_read_write_tasks<
    TS: Serialize + Send + Sync + 'static,
    TR: for<'a> Deserialize<'a> + std::fmt::Debug + Send + Sync + 'static,
>(
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> (mpsc::UnboundedSender<TS>, mpsc::UnboundedReceiver<TR>) {
    let (mut tx, mut rx) = socket.split();
    let (request_tx, mut request_rx) = mpsc::unbounded_channel();
    let (response_tx, response_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(value) = request_rx.recv().await {
            if let Err(err) = tx
                .send(Message::Binary(serde_cbor::to_vec(&value).unwrap()))
                .await
            {
                tracing::error!("Failed to send to websocket: {err}");
                return;
            }
        }
    });
    tokio::spawn(async move {
        loop {
            tokio::select!(
                 value = rx.next() => {
                    match value {
                        Some(Ok(Message::Binary(value))) => {
                            let a = serde_cbor::from_slice(&value).unwrap();
                            response_tx.send(a).unwrap();
                        }
                        Some(Ok(other)) => tracing::error!("Unexpected message {other:?}"),
                        Some(Err(err)) => tracing::error!("Failed to receive message from websocket {err:?}"),
                        None => return,
                    }
                }
                _ = response_tx.closed() => {
                    return;
                }
            )
        }
    });

    (request_tx, response_rx)
}

// TODO: axum should expose way to get tungstenite types so we can avoid this duplication
pub mod axum {
    use axum::extract::ws::{Message, WebSocket};
    use futures_util::{SinkExt, StreamExt};
    use serde::{Deserialize, Serialize};
    use tokio::sync::mpsc;

    pub async fn spawn_read_write_tasks<
        TS: Serialize + Send + Sync + 'static,
        TR: for<'a> Deserialize<'a> + std::fmt::Debug + Send + Sync + 'static,
    >(
        socket: WebSocket,
    ) -> (mpsc::UnboundedSender<TS>, mpsc::UnboundedReceiver<TR>) {
        let (mut tx, mut rx) = socket.split();
        let (request_tx, mut request_rx) = mpsc::unbounded_channel();
        let (response_tx, response_rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(value) = request_rx.recv().await {
                if let Err(err) = tx
                    .send(Message::Binary(serde_cbor::to_vec(&value).unwrap()))
                    .await
                {
                    tracing::error!("Failed to send to websocket: {err}");
                    return;
                }
            }
        });
        tokio::spawn(async move {
            loop {
                tokio::select!(
                     value = rx.next() => {
                        match value {
                            Some(Ok(Message::Binary(value))) => {
                                let a = serde_cbor::from_slice(&value).unwrap();
                                response_tx.send(a).unwrap();
                            }
                            Some(Ok(other)) => tracing::error!("Unexpected message {other:?}"),
                            Some(Err(err)) => {
                                // TODO: ughhh seriously axum??
                                let a = format!("{:?}",err);
                                if !a.contains("ResetWithoutClosingHandshake") {
                                    tracing::error!("Failed to receive message from websocket: {err:?}");
                                }
                            }
                            None => return,
                        }
                    }
                    _ = response_tx.closed() => {
                        return;
                    }
                )
            }
        });

        (request_tx, response_rx)
    }
}
