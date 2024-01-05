use std::collections::HashMap;
use std::process::Stdio;

use futures::stream::SplitSink;
use futures::{SinkExt as _, StreamExt as _};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt as _, BufReader};
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
    add_pending_command: mpsc::Sender<(u64, oneshot::Sender<String>)>,
    // TODO FIXME browsing context specific events?
}

impl WebDriver {
    /// Creates a new [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) connection.
    /// ## Errors
    /// Returns an error if the `WebSocket` connection fails.
    pub async fn new() -> Result<Self, tokio_tungstenite::tungstenite::Error> {
        let tmp_dir = tempdir()?;

        let mut child = tokio::process::Command::new("firefox")
            .kill_on_drop(true)
            .args([
                "--profile",
                &tmp_dir.path().to_string_lossy(),
                "--no-remote",
                "--new-instance",
                //"--headless",
                "--remote-debugging-port",
                "0",
            ])
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn command");

        let stderr = child
            .stderr
            .take()
            .expect("child did not have a handle to stdout");

        let mut reader = BufReader::new(stderr).lines();

        // Ensure the child process is spawned in the runtime so it can
        // make progress on its own while we await for any output.
        tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .expect("child process encountered an error");

            println!("child status was: {status}");
        });

        let mut port = None;
        while let Some(line) = reader.next_line().await? {
            eprintln!("{line}");
            if let Some(p) = line.strip_prefix("WebDriver BiDi listening on ws://127.0.0.1:") {
                port = Some(p.parse::<u16>().unwrap());
                break;
            }
        }

        let Some(port) = port else {
            panic!("failed to retrieve port");
        };

        tokio::spawn(async move {
            while let Some(line) = reader.next_line().await? {
                eprintln!("{line}");
            }
            Ok::<(), std::io::Error>(())
        });

        let (stream, _response) =
            tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/session")).await?;
        let (sink, mut stream) = stream.split();

        let (tx, mut rx) = mpsc::channel::<(u64, oneshot::Sender<String>)>(100);

        tokio::spawn(async move {
            let mut pending_requests = HashMap::<u64, oneshot::Sender<String>>::new();

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
            add_pending_command: tx,
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
            WebDriverBiDiLocalEndMessage::Event(event) => todo!("{event:?}"),
        }
    }

    pub(crate) async fn send_command<ResultData: DeserializeOwned>(
        &mut self,
        command_data: WebDriverBiDiRemoteEndCommandData,
    ) -> Result<ResultData, tokio_tungstenite::tungstenite::Error> {
        let (tx, rx) = oneshot::channel();

        let id: u64 = self.current_id;
        self.current_id += 1;
        self.add_pending_command.send((id, tx)).await.unwrap();

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
            .send_command(WebDriverBiDiRemoteEndCommandData::Session(
                session::Command::New(crate::session::new::Command {
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
