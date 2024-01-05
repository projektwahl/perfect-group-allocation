use std::collections::HashMap;

use futures::stream::SplitSink;
use futures::{SinkExt as _, StreamExt as _};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::webdriver_session::WebDriverSession;
use crate::{
    session, WebDriverBiDiLocalEndCommandResponse, WebDriverBiDiLocalEndMessage,
    WebDriverBiDiRemoteEndCommand, WebDriverBiDiRemoteEndCommandData,
};

/// <https://w3c.github.io/webdriver-bidi>
#[derive(Debug)]
pub struct WebDriver {
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    current_id: u64,
    add_pending_request: mpsc::Sender<(u64, oneshot::Sender<String>)>,
}

impl WebDriver {
    /// Creates a new [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) connection.
    /// ## Errors
    /// Returns an error if the `WebSocket` connection fails.
    pub async fn new() -> Result<Self, tokio_tungstenite::tungstenite::Error> {
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
        let parsed_message: WebDriverBiDiLocalEndMessage<Value> =
            serde_json::from_str(&message).unwrap();
        match parsed_message {
            WebDriverBiDiLocalEndMessage::CommandResponse(parsed_message) => {
                pending_requests
                    .remove(&parsed_message.id)
                    .unwrap()
                    .send(message)
                    .unwrap();
            }
            WebDriverBiDiLocalEndMessage::ErrorResponse(error) => {
                println!("error {error:#?}"); // TODO FIXME propage to command if it has an id.
            }
            WebDriverBiDiLocalEndMessage::Event(_) => todo!(),
        }
    }

    pub(crate) async fn send_command<ResultData: DeserializeOwned>(
        &mut self,
        command_data: WebDriverBiDiRemoteEndCommandData,
    ) -> Result<ResultData, tokio_tungstenite::tungstenite::Error> {
        let (tx, rx) = oneshot::channel();

        let id: u64 = self.current_id;
        self.current_id += 1;
        self.add_pending_request.send((id, tx)).await.unwrap();

        self.sink
            .send(Message::Text(
                serde_json::to_string(&WebDriverBiDiRemoteEndCommand {
                    id,
                    command_data,
                    extensible: Value::Null,
                })
                .unwrap(),
            ))
            .await?;
        self.sink.flush().await?;

        let received = rx.await.unwrap();
        let parsed: WebDriverBiDiLocalEndMessage<ResultData> =
            serde_json::from_str(&received).expect(&received);
        match parsed {
            WebDriverBiDiLocalEndMessage::CommandResponse(
                WebDriverBiDiLocalEndCommandResponse {
                    result: result_data,
                    ..
                },
            ) => Ok(result_data),
            WebDriverBiDiLocalEndMessage::ErrorResponse(error_response) => {
                panic!("{error_response:?}");
            }
            WebDriverBiDiLocalEndMessage::Event(_) => {
                unreachable!("command should never get an event as response");
            }
        }
    }

    pub async fn session_new(
        mut self,
    ) -> Result<WebDriverSession, tokio_tungstenite::tungstenite::Error> {
        let result: crate::session::new::Result = self
            .send_command(WebDriverBiDiRemoteEndCommandData::SessionCommand(
                session::Command::SessionNew(crate::session::new::CommandType {
                    params: session::new::Parameters {
                        capabilities: session::new::CapabilitiesRequest {},
                    },
                }),
            ))
            .await?;
        println!("{result:?}");
        Ok(WebDriverSession {
            session_id: result.session_id,
            driver: self,
        })
    }
}
