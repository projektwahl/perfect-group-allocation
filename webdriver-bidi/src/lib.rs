use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// https://w3c.github.io/webdriver-bidi
pub struct WebDriverBiDi {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebDriverBiDi {
    /// Creates a new [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) connection.
    /// ## Errors
    /// Returns an error if the `WebSocket` connection fails.
    pub async fn new() -> Result<Self, tokio_tungstenite::tungstenite::Error> {
        // firefox --profile /tmp/a --new-instance --remote-debugging-port 9222

        let (stream, response) =
            tokio_tungstenite::connect_async("ws://127.0.0.1:9222/session").await?;

        Ok(Self { stream })
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

#[cfg(test)]
mod tests {
    use futures::{SinkExt as _, StreamExt as _};
    use tokio_tungstenite::tungstenite::Message;

    use crate::WebDriverBiDiMessage;

    #[tokio::test]
    async fn it_works() -> Result<(), tokio_tungstenite::tungstenite::Error> {
        stream
            .send(Message::Text(
                serde_json::to_string(&WebDriverBiDiMessage {
                    id: todo!(),
                    method: todo!(),
                    params: todo!(),
                })
                .unwrap(),
            ))
            .await?;
        stream.flush().await?;

        while let Some(msg) = stream.next().await {
            let msg = msg?;
            println!("{msg:?}");
        }

        Ok(())
    }
}
