use std::collections::HashMap;
use std::fmt::Debug;

use futures::{SinkExt as _, StreamExt as _};
use serde::Serialize;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{
    browsing_context, session, WebDriverBiDiLocalEndCommandResponse, WebDriverBiDiLocalEndMessage,
    WebDriverBiDiLocalEndMessageErrorResponse, WebDriverBiDiRemoteEndCommand,
};

macro_rules! magic {
    (pub enum { $(#[doc = $doc:expr] $variant:ident($tag:literal $($command:ident)::+)),* }) => {
        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug)]
        pub enum SendCommand {
            $(#[doc = $doc] $variant($($command)::*::Command, oneshot::Sender<oneshot::Receiver<$($command)::*::Result>>),)*
        }

        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug)]
        pub enum RespondCommand {
            $(#[doc = $doc] $variant(oneshot::Sender<$($command)::*::Result>),)*
        }

        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(tag = "method")]
        pub enum WebDriverBiDiRemoteEndCommandData {
            $(
                #[doc = $doc]
                #[serde(rename = $tag)]
                $variant($($command)::*::Command)
            ,)*
        }

        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(tag = "method")]
        pub enum ResultData {
            $(
                #[doc = $doc]
                #[serde(rename = $tag)]
                $variant($($command)::*::Result)
            ,)*
        }

        async fn handle_command(this: &mut WebDriverHandler, input: SendCommand) -> crate::result::Result<()> {
            match input {
                $(
                    SendCommand::$variant(command, sender) => {
                        this.handle_command_internal(command, sender, RespondCommand::$variant).await
                    }
                ),*
            }
        }

        fn send_response(result: Value, respond_command: RespondCommand) -> crate::result::Result<()> {
            match (respond_command) {
                $(
                    RespondCommand::$variant(respond_command) => {
                        respond_command
                            .send(serde_path_to_error::deserialize(result)
                                .map_err(crate::result::Error::ParseReceivedWithPath)?)
                            .map_err(|_| crate::result::Error::CommandCallerExited)
                    }
                ),*
            }
        }
    };
}

magic! {
    pub enum {
        /// https://w3c.github.io/webdriver-bidi/#command-session-new
        SessionNew("session.new" session::new),
        /// https://w3c.github.io/webdriver-bidi/#command-session-end
        SessionEnd("session.end" session::end),
        /// https://w3c.github.io/webdriver-bidi/#command-session-subscribe
        SessionSubscribe("session.subscribe" session::subscribe),
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
        BrowsingContextGetTree("browsingContext.getTree" browsing_context::get_tree),
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
        BrowsingContextNavigate("browsingContext.navigate" browsing_context::navigate)
    }
}

pub struct WebDriverHandler {
    id: u64,
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    receive_command: mpsc::Receiver<SendCommand>,
    pending_commands: HashMap<u64, RespondCommand>,
}

impl WebDriverHandler {
    pub async fn handle(
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        receive_command: mpsc::Receiver<SendCommand>,
    ) {
        let mut this = Self {
            id: 0,
            stream,
            receive_command,
            pending_commands: HashMap::default(),
        };
        this.handle_internal().await;
    }

    async fn handle_internal(&mut self) {
        loop {
            tokio::select! {
                // TODO FIXME is this cancel safe?
                message = self.stream.next() => {
                    match message {
                        Some(Ok(Message::Text(message))) => {
                            if let Err(error) = self.handle_message(&message) {
                                eprintln!("error {error:?}");
                            }
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
                    if let Err(error) = handle_command(self, command_session_new).await {
                        eprintln!("error {error:?}");
                    }
                }
            }
        }
        println!("handle closed");
    }

    async fn handle_command_internal<C: Serialize + Debug, R>(
        &mut self,
        command_data: C,
        sender: oneshot::Sender<oneshot::Receiver<R>>,
        respond_command_constructor: impl FnOnce(oneshot::Sender<R>) -> RespondCommand,
    ) -> crate::result::Result<()> {
        self.id += 1;

        let (tx, rx) = oneshot::channel();
        self.pending_commands
            .insert(self.id, respond_command_constructor(tx));

        let string = serde_json::to_string(&WebDriverBiDiRemoteEndCommand {
            id: self.id,
            command_data,
        })
        .unwrap();

        println!("{string}");

        // starting from here this could be done asynchronously
        // TODO FIXME I don't think we need the flushing requirement here specifically. maybe flush if no channel is ready or something like that
        self.stream.send(Message::Text(string)).await?;

        sender
            .send(rx)
            .map_err(|_| crate::result::Error::CommandCallerExited)?;

        Ok(())
    }

    fn handle_message(&mut self, message: &str) -> crate::result::Result<()> {
        println!("{message}");
        let jd = &mut serde_json::Deserializer::from_str(message);
        let parsed_message: WebDriverBiDiLocalEndMessage<Value> =
            serde_path_to_error::deserialize(jd)
                .map_err(crate::result::Error::ParseReceivedWithPath)?;
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
                    id: _,
                    error,
                    message: _,
                    stacktrace: _,
                    extensible: _,
                },
            ) => {
                // TODO FIXME we need a id -> channel mapping bruh
                eprintln!("error {error:#?}"); // TODO FIXME propage to command if it has an id.

                Ok(())
            }
            WebDriverBiDiLocalEndMessage::Event(event) => todo!("{event:?}"),
        }
    }
}
