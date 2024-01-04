#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
use std::collections::HashMap;

use futures::stream::SplitSink;
use futures::{SinkExt as _, StreamExt as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// <https://w3c.github.io/webdriver-bidi>
pub struct WebDriverBiDi {
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    current_id: u64,
    add_pending_request: mpsc::Sender<(u64, oneshot::Sender<String>)>,
}

impl WebDriverBiDi {
    /// Creates a new [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) connection.
    /// ## Errors
    /// Returns an error if the `WebSocket` connection fails.
    pub async fn new() -> Result<Self, tokio_tungstenite::tungstenite::Error> {
        // firefox --profile /tmp/a --new-instance --remote-debugging-port 9222

        let (stream, _response) =
            tokio_tungstenite::connect_async("ws://127.0.0.1:9222/session").await?;
        let (sink, mut stream) = stream.split();

        let (tx, mut rx) = mpsc::channel::<(u64, oneshot::Sender<String>)>(100);

        tokio::spawn(async move {
            let mut pending_requests = HashMap::<u64, oneshot::Sender<String>>::new();

            #[expect(clippy::redundant_pub_crate, reason = "tokio::select!")]
            loop {
                tokio::select! {
                    message = stream.next() => {
                        match message {
                            Some(Ok(Message::Text(message))) => {
                                Self::handle_message(&mut pending_requests, message);
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
                    pending_request = rx.recv() => {
                        if let Some(pending_request) = pending_request {
                            pending_requests.insert(pending_request.0, pending_request.1);
                        }
                    }
                }
            }
        });

        Ok(Self {
            sink,
            current_id: 0,
            add_pending_request: tx,
        })
    }

    fn handle_message(
        pending_requests: &mut HashMap<u64, oneshot::Sender<String>>,
        message: String,
    ) {
        let parsed_message: WebDriverBiDiLocalEndMessage = serde_json::from_str(&message).unwrap();
        match parsed_message {
            WebDriverBiDiLocalEndMessage::CommandResponse(parsed_message) => {
                pending_requests
                    .remove(&parsed_message.id)
                    .unwrap()
                    .send(message)
                    .unwrap();
            }
            WebDriverBiDiLocalEndMessage::ErrorResponse(error) => {
                println!("error {error:#?}");
            }
            WebDriverBiDiLocalEndMessage::Event(_) => todo!(),
        }
    }

    async fn send_command(
        &mut self,
        command: WebDriverBiDiCommand,
    ) -> Result<String, tokio_tungstenite::tungstenite::Error> {
        let (tx, rx) = oneshot::channel();

        self.current_id += 1;
        self.add_pending_request
            .send((self.current_id, tx))
            .await
            .unwrap();

        self.sink
            .send(Message::Text(serde_json::to_string(&command).unwrap()))
            .await?;
        self.sink.flush().await?;

        Ok(rx.await.unwrap())
    }

    pub async fn create_session(
        &mut self,
    ) -> Result<SessionNewResult, tokio_tungstenite::tungstenite::Error> {
        Ok(serde_json::from_str(
            &self
                .send_command(WebDriverBiDiCommand::SessionNew(SessionNewParameters {
                    capabilities: SessionCapabilitiesRequest {},
                }))
                .await?,
        )
        .unwrap())
    }
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebDriverBiDiLocalEndMessage {
    #[serde(rename = "success")]
    CommandResponse(WebDriverBiDiLocalEndCommandResponse),
    #[serde(rename = "error")]
    ErrorResponse(WebDriverBiDiLocalEndMessageErrorResponse),
    #[serde(rename = "event")]
    Event(WebDriverBiDiLocalEndEvent),
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiLocalEndCommandResponse {
    id: u64,
    result: Value,
    // Extensible
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiLocalEndEvent {
    id: u64,
    // TODO EventData
    // Extensible
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiLocalEndMessageErrorResponse {
    id: Option<u64>,
    error: String,
    message: String,
    stacktrace: Option<String>,
    // Extensible
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebDriverBiDiCommand {
    /// https://w3c.github.io/webdriver-bidi/#command-session-new
    SessionNew(SessionNewParameters),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionNewParameters {
    capabilities: SessionCapabilitiesRequest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionCapabilitiesRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionNewResult {
    sessionId: String,
    capabilities: SessionNewResultCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionNewResultCapabilities {
    acceptInsecureCerts: bool,
    browserName: String,
    browserVersion: String,
    platformName: String,
    setWindowRect: bool,
    //proxy: Option<session.ProxyConfiguration>,
    //webSocketUrl: Option<text / true>,
    //Extensible
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn it_works() -> Result<(), tokio_tungstenite::tungstenite::Error> {
        Ok(())
    }
}
