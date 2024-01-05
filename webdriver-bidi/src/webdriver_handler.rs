use std::collections::HashMap;

use futures::{SinkExt as _, StreamExt as _};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{session, WebDriverBiDiLocalEndMessage, WebDriverBiDiRemoteEndCommand};

pub struct WebDriverHandler {
    id: u64,
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    command_session_new_rx: mpsc::Receiver<(
        session::new::Command,
        oneshot::Sender<oneshot::Receiver<session::new::Result>>,
    )>,
    pending_session_new: HashMap<u64, oneshot::Sender<session::new::Result>>,
}

impl WebDriverHandler {
    pub async fn new(
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        command_session_new_rx: mpsc::Receiver<(
            session::new::Command,
            oneshot::Sender<oneshot::Receiver<session::new::Result>>,
        )>,
    ) {
        let mut this = Self {
            id: 0,
            stream,
            command_session_new_rx,
            pending_session_new: Default::default(),
        };
        this.handle().await;
    }

    pub async fn handle(&mut self) {
        loop {
            tokio::select! {
                // TODO FIXME is this cancel safe?
                message = self.stream.next() => {
                    match message {
                        Some(Ok(Message::Text(message))) => {
                           self.handle_message(message);
                        }
                        Some(Ok(message)) => {
                            println!("Unknown message: {message:#?}");
                        }
                        Some(Err(error)) => println!("Error {error:#?}"),
                        None => {
                            println!("connection closed");
                            break;
                        }
                    }
                }
                Some(command_session_new) = self.command_session_new_rx.recv() => {
                    self.handle_command_session_new(command_session_new).await;
                }
            }
        }
    }

    async fn handle_command_session_new(
        &mut self,
        input: (
            session::new::Command,
            oneshot::Sender<oneshot::Receiver<session::new::Result>>,
        ),
    ) -> crate::result::Result<()> {
        self.id += 1;

        let (tx, rx) = oneshot::channel();
        self.pending_session_new.insert(self.id, tx);

        // starting from here this could be done asynchronously
        // TODO FIXME I don't think we need the flushing requirement here specifically. maybe flush if no channel is ready or something like that
        self.stream
            .send(Message::Text(
                serde_json::to_string(&WebDriverBiDiRemoteEndCommand {
                    id: self.id,
                    command_data: input.0,
                })
                .unwrap(),
            ))
            .await?;

        input
            .1
            .send(rx)
            .map_err(|_| crate::result::Error::CommandCallerExited)?;

        Ok(())
    }

    fn handle_message(&mut self, message: String) {
        let parsed_message: WebDriverBiDiLocalEndMessage<Value> = serde_json::from_str(&message)?;
        match parsed_message {
            WebDriverBiDiLocalEndMessage::CommandResponse(parsed_message) => {
                pending_requests.remove(&parsed_message.id)?.send(message)?;
            }
            WebDriverBiDiLocalEndMessage::ErrorResponse(error) => {
                println!("error {error:#?}"); // TODO FIXME propage to command if it has an id.
            }
            WebDriverBiDiLocalEndMessage::Event(event) => todo!("{event:?}"),
        }
    }
}
