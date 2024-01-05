use std::collections::HashMap;
use std::process::Stdio;

use futures::stream::SplitSink;
use futures::{Future, SinkExt as _, StreamExt as _};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::browsing_context::BrowsingContext;
use crate::webdriver_session::WebDriverSession;
use crate::{
    log, session, WebDriverBiDiLocalEndCommandResponse, WebDriverBiDiLocalEndMessage,
    WebDriverBiDiRemoteEndCommand, WebDriverBiDiRemoteEndCommandData,
};

/// <https://w3c.github.io/webdriver-bidi>
// TODO FIXME make usable from multiple threads
// TODO FIXME implement pipelining
#[derive(Debug, Clone)]
pub struct WebDriver {
    /// send a session new command
    command_session_new: mpsc::Sender<(
        crate::session::new::Command,
        oneshot::Sender<oneshot::Receiver<crate::session::new::Result>>,
    )>,
    // send a subscribe command for global log messages and receive the subscription channel.
    // when we have received the subscription channel we can be sure that the command has been sent (for ordering purposes).
    // if you don't care about the relative ordering of the commands you can join! etc. this future.
    //event_handlers_log: mpsc::Sender<((), oneshot::Sender<broadcast::Receiver<log::Entry>>)>,
    // channel to receive browsing context specific log messages
    //event_handlers_browsing_context_log: mpsc::Sender<(
    //    BrowsingContext,
    //    oneshot::Sender<broadcast::Receiver<log::Entry>>,
    //)>,
    // wait we want to subscribe and unsubscribe per subscription and also then end the subscriber.
    // how do we implement this?. I think we unsubscribe by dropping the receiver and
    // then the sender realizes this and unsubscribes by itself?
    // send a subscribe command and then receive the subscription channel
    //event_handlers_browsing_context:
    //    mpsc::Sender<(String, oneshot::Sender<broadcast::Receiver<String>>)>,
}

impl WebDriver {
    /// Creates a new [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) connection.
    /// ## Errors
    /// Returns an error if the `WebSocket` connection fails.
    pub async fn new() -> Result<Self, crate::result::Error> {
        let tmp_dir = tempdir().map_err(crate::result::Error::TmpDirCreateError)?;

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
            .map_err(crate::result::Error::SpawnBrowserError)?;

        let stderr = child.stderr.take().unwrap();

        let mut reader = BufReader::new(stderr).lines();

        // Ensure the child process is spawned in the runtime so it can
        // make progress on its own while we await for any output.
        tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .map_err(crate::result::Error::FailedToRunBrowser)?;

            println!("child status was: {status}");

            Ok::<(), crate::result::Error>(())
        });

        let mut port = None;
        while let Some(line) = reader
            .next_line()
            .await
            .map_err(crate::result::Error::ReadBrowserStderr)?
        {
            eprintln!("{line}");
            if let Some(p) = line.strip_prefix("WebDriver BiDi listening on ws://127.0.0.1:") {
                port = Some(
                    p.parse::<u16>()
                        .map_err(crate::result::Error::PortDetectError)?,
                );
                break;
            }
        }

        let Some(port) = port else {
            return Err(crate::result::Error::PortNotFound);
        };

        tokio::spawn(async move {
            while let Some(line) = reader.next_line().await? {
                eprintln!("{line}");
            }
            Ok::<(), std::io::Error>(())
        });

        let (mut stream, _response) =
            tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/session")).await?;

        let (command_session_new, mut command_session_new_rx) = mpsc::channel(1);
        //let (event_handlers_log, event_handlers_log_rx) = mpsc::channel(10);

        tokio::spawn(async move {
            let mut id = 0;
            let mut pending_session_new =
                HashMap::<u64, oneshot::Sender<session::new::Result>>::new();

            loop {
                tokio::select! {
                    message = stream.next() => {
                        match message {
                            Some(Ok(Message::Text(message))) => {
                               //Self::handle_message(&mut pending_requests, message);
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
                    Some(command_session_new) = command_session_new_rx.recv() => {
                        Self::handle_command_session_new(&mut id, &mut stream, &mut pending_session_new, command_session_new).await;
                    }
                }
            }
        });

        Ok(Self {
            command_session_new,
        })
    }

    async fn handle_command_session_new(
        id: &mut u64,
        stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
        pending_session_new: &mut HashMap<u64, oneshot::Sender<session::new::Result>>,
        input: (
            session::new::Command,
            oneshot::Sender<oneshot::Receiver<session::new::Result>>,
        ),
    ) -> crate::result::Result<()> {
        stream
            .send(Message::Text(
                serde_json::to_string(&WebDriverBiDiRemoteEndCommand {
                    id: *id,
                    command_data: input.0,
                })
                .unwrap(),
            ))
            .await?;

        let (tx, rx) = oneshot::channel();
        pending_session_new.insert(*id, tx);

        *id += 1;

        input
            .1
            .send(rx)
            .map_err(|_| crate::result::Error::CommandCallerExited)?;

        Ok(())
    }

    fn handle_message(
        pending_requests: &mut HashMap<u64, oneshot::Sender<String>>,
        message: String,
    ) {
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

    pub fn session_new<'a>(
        &self,
    ) -> impl Future<
        Output = crate::result::Result<
            impl Future<Output = crate::result::Result<WebDriverSession>>,
        >,
    > {
        async {
            let result = self
                .send_command(
                    &self.command_session_new,
                    crate::session::new::Command {
                        params: session::new::Parameters {
                            capabilities: session::new::CapabilitiesRequest {},
                        },
                    },
                )
                .await?;
            Ok(async {
                let result = result.await?;
                Ok(WebDriverSession {
                    session_id: result.session_id,
                    driver: self.clone(),
                })
            })
        }
    }

    pub(crate) fn send_command<'a: 'b, 'b, C: 'static, R: 'static>(
        &'a self,
        queue: &'b mpsc::Sender<(C, oneshot::Sender<oneshot::Receiver<R>>)>,
        command: C,
    ) -> impl Future<
        Output = crate::result::Result<impl Future<Output = crate::result::Result<R>> + 'a>,
    > + 'b {
        let (tx, rx) = oneshot::channel();
        let sending = queue.send((command, tx));
        async {
            sending
                .await
                .map_err(|_| crate::result::Error::CommandTaskExited)?;

            // when we received the final receiver, we can be sure that our command got handled and is ordered before all commands that we sent afterwards.
            let rx = rx
                .await
                .map_err(|_| crate::result::Error::CommandTaskExited)?;

            Ok(async {
                let result = rx
                    .await
                    .map_err(|_| crate::result::Error::CommandTaskExited)?;
                Ok(result)
            })
        }
    }
}
