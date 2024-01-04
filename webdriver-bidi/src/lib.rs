use core::any::Any;
use std::collections::HashMap;

use futures::stream::SplitSink;
use futures::{SinkExt as _, StreamExt as _};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// https://w3c.github.io/webdriver-bidi
pub struct WebDriverBiDi {
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    // TODO FIXMe maybe use an mpsc to send them to the worker
    pending_requests: HashMap<u64, oneshot::Sender<String>>,
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

        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(Message::Text(message)) => {
                        let message: WebDriverBiDiMessage = serde_json::from_str(&message).unwrap();
                    }
                    Ok(message) => {
                        println!("Unknown message: {message:?}")
                    }
                    Err(error) => println!("Error {error:?}"),
                }
            }
        });

        Ok(Self {
            sink,
            pending_requests: HashMap::new(),
        })
    }

    async fn send_command(
        &mut self,
        command: WebDriverBiDiCommand,
    ) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        let (tx, rx) = oneshot::channel();

        self.pending_requests.insert(1, tx);
        self.sink
            .send(Message::Text(serde_json::to_string(&command).unwrap()))
            .await?;
        self.sink.flush().await?;

        Ok(rx.await.unwrap())
    }

    pub async fn create_session(&mut self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        self.send_command(WebDriverBiDiCommand::SessionNew(SessionNewParameters {
            capabilities: SessionCapabilitiesRequest {},
        }))
        .await
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
