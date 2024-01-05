use std::collections::HashMap;

use futures::{SinkExt as _, StreamExt as _};
use serde::Serialize;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{
    session, CommandResultPair, ResultData, WebDriverBiDiLocalEndCommandResponse,
    WebDriverBiDiLocalEndMessage, WebDriverBiDiLocalEndMessageErrorResponse,
    WebDriverBiDiRemoteEndCommand,
};

macro_rules! magic {
    (pub enum { $(#[doc = $doc:expr] $variant:ident($($command:ident)::+)),* }) => {
        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        pub enum SendCommand {
            $(#[doc = $doc] $variant($($command)::*::Command, oneshot::Sender<oneshot::Receiver<$($command)::*::Result>>),)*
        }

        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        pub enum RespondCommand {
            $(#[doc = $doc] $variant(oneshot::Sender<$($command)::*::Result>),)*
        }

        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(untagged)]
        pub enum WebDriverBiDiRemoteEndCommandData {
            $(#[doc = $doc] $variant($($command)::*::Command),)*
        }

        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(untagged)]
        pub enum ResultData {
            $(#[doc = $doc] $variant($($command)::*::Result),)*
        }

        async fn handle_command(this: &mut WebDriverHandler, input: SendCommand) -> crate::result::Result<()> {
            match input {
                $(
                    SendCommand::$variant(command, sender) => {
                        this.handle_command_internal(command, sender).await
                    }
                ),*
            }
        }

        fn send_response(result: ResultData, respond_command: RespondCommand) -> crate::result::Result<()> {
            match (result, respond_command) {
                $(
                    (ResultData::$variant(result), RespondCommand::$variant(respond_command)) => {
                        respond_command
                            .send(result)
                            .map_err(|_| crate::result::Error::CommandCallerExited)
                    }
                ),*
                _ => panic!(),
            }
        }
    };
}

magic! {
    pub enum {
        /// https://w3c.github.io/webdriver-bidi/#command-session-new
        SessionNew(session::new),
        /// https://w3c.github.io/webdriver-bidi/#command-session-end
        SessionEnd(session::end),
        /// https://w3c.github.io/webdriver-bidi/#command-session-subscribe
        SessionSubscribe(session::subscribe),
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
        BrowsingContextGetTree(browsing_context::get_tree),
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
        BrowsingContextNavigate(browsing_context::navigate)
    }
}

pub struct WebDriverHandler {
    id: u64,
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    receive_command: mpsc::Receiver<SendCommand>,
    pending_commands: HashMap<u64, RespondCommand>,
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
            pending_commands: Default::default(),
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
                Some(command_session_new) = self.receive_command.recv() => {
                    self.handle_command(command_session_new).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, input: SendCommand) -> crate::result::Result<()> {
        match input {
            SendCommand::SessionNew(command, sender) => {
                self.handle_command_internal(command, sender).await
            }
            SendCommand::SessionEnd(command, sender) => {
                self.handle_command_internal(command, sender).await
            }
            SendCommand::SessionSubscribe(command, sender) => {
                self.handle_command_internal(command, sender).await
            }
            SendCommand::BrowsingContextGetTree(command, sender) => {
                self.handle_command_internal(command, sender).await
            }
            SendCommand::BrowsingContextNavigate(command, sender) => {
                self.handle_command_internal(command, sender).await
            }
        }
    }

    async fn handle_command_internal<C: Serialize, R>(
        &mut self,
        command_data: C,
        sender: oneshot::Sender<oneshot::Receiver<R>>,
        respond_command_constructor: impl FnOnce(oneshot::Sender<R>) -> RespondCommand,
    ) -> crate::result::Result<()> {
        self.id += 1;

        let (tx, rx) = oneshot::channel();
        self.pending_commands
            .insert(self.id, respond_command_constructor(tx));

        // starting from here this could be done asynchronously
        // TODO FIXME I don't think we need the flushing requirement here specifically. maybe flush if no channel is ready or something like that
        self.stream
            .send(Message::Text(
                serde_json::to_string(&WebDriverBiDiRemoteEndCommand {
                    id: self.id,
                    command_data,
                })
                .unwrap(),
            ))
            .await?;

        sender
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
            ) => {
                let respond_command = self
                    .pending_commands
                    .remove(&id)
                    .ok_or(crate::result::Error::ResponseWithoutRequest(id))?;

                send_response(result, respond_command)
            }
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
