use core::any::Any;
use std::collections::HashMap;

use futures::stream::SplitSink;
use futures::{SinkExt as _, StreamExt as _};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// https://w3c.github.io/webdriver-bidi
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

        let (stream, response) =
            tokio_tungstenite::connect_async("ws://127.0.0.1:9222/session").await?;
        let (sink, mut stream) = stream.split();

        let (tx, mut rx) = mpsc::channel::<(u64, oneshot::Sender<String>)>(100);

        tokio::spawn(async move {
            let pending_requests = HashMap::<u64, oneshot::Sender<String>>::new();

            loop {
                tokio::select! {
                    message = stream.next() => {
                        match message {
                            Some(Ok(Message::Text(message))) => {
                                let message: WebDriverBiDiMessage = serde_json::from_str(&message).unwrap();
                                pending_requests.remove(&message.id).unwrap().send(message);
                            }
                            Some(Ok(message)) => {
                                println!("Unknown message: {message:?}")
                            }
                            Some(Err(error)) => println!("Error {error:?}"),
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

    async fn send_command(
        &mut self,
        command: WebDriverBiDiCommand,
    ) -> Result<String, tokio_tungstenite::tungstenite::Error> {
        let (tx, rx) = oneshot::channel();

        self.current_id += 1;
        self.add_pending_request.send((self.current_id, tx)).await;

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

// https://w3c.github.io/webdriver-bidi/#handle-an-incoming-message
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiMessage {
    id: u64,
    method: String,
    params: String,
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
    use futures::{SinkExt as _, StreamExt as _};
    use tokio_tungstenite::tungstenite::Message;

    use crate::WebDriverBiDiMessage;

    #[tokio::test]
    async fn it_works() -> Result<(), tokio_tungstenite::tungstenite::Error> {
        Ok(())
    }
}
