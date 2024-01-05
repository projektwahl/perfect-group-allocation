use std::collections::HashMap;

use futures::{SinkExt as _, StreamExt as _};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::webdriver::SendCommand;
use crate::{
    session, ResultData, WebDriverBiDiLocalEndCommandResponse, WebDriverBiDiLocalEndMessage,
    WebDriverBiDiLocalEndMessageErrorResponse, WebDriverBiDiRemoteEndCommand,
};

pub struct WebDriverHandler {
    id: u64,
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    receive_command: mpsc::Receiver<SendCommand>,

    pending_session_new: HashMap<u64, oneshot::Sender<session::new::Result>>,
    pending_session_end: HashMap<u64, oneshot::Sender<session::end::Result>>,
}

impl WebDriverHandler {
    pub async fn new(
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        receive_command: mpsc::Receiver<SendCommand>,
    ) {
        let mut this = Self {
            id: 0,
            stream,
            receive_command,
            pending_session_new: Default::default(),
            pending_session_end: Default::default(),
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
                Some(command_session_end) = self.command_session_end_rx.recv() => {
                    self.handle_command_session_end(command_session_end).await;
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

    fn handle_message(&mut self, message: String) -> crate::result::Result<()> {
        let parsed_message: WebDriverBiDiLocalEndMessage<ResultData> =
            serde_json::from_str(&message).map_err(crate::result::Error::ParseReceived)?;
        match parsed_message {
            WebDriverBiDiLocalEndMessage::CommandResponse(
                WebDriverBiDiLocalEndCommandResponse { id, result },
            ) => match result {
                ResultData::Session(session::Result::New(new)) => self
                    .pending_session_new
                    .remove(&id)
                    .ok_or(crate::result::Error::ResponseWithoutRequest(id))?
                    .send(new)
                    .map_err(|_| crate::result::Error::CommandCallerExited),
                ResultData::BrowsingContext(_) => todo!(),
            },
            WebDriverBiDiLocalEndMessage::ErrorResponse(
                WebDriverBiDiLocalEndMessageErrorResponse {
                    id,
                    error,
                    message,
                    stacktrace,
                    extensible,
                },
            ) => {
                // TODO FIXME we need a id -> channel mapping bruh
                println!("error {error:#?}"); // TODO FIXME propage to command if it has an id.

                Ok(())
            }
            WebDriverBiDiLocalEndMessage::Event(event) => todo!("{event:?}"),
        }
    }
}
