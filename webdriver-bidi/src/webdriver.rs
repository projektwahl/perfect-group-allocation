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
use crate::webdriver_handler::{SendCommand, WebDriverHandler};
use crate::webdriver_session::WebDriverSession;
use crate::{
    log, session, WebDriverBiDiLocalEndCommandResponse, WebDriverBiDiLocalEndMessage,
    WebDriverBiDiRemoteEndCommand,
};

/// <https://w3c.github.io/webdriver-bidi>
#[derive(Debug, Clone)]
pub struct WebDriver {
    /// send a command
    send_command: mpsc::Sender<SendCommand>,
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

        tokio::spawn(WebDriverHandler::new(stream, command_session_new_rx));

        Ok(Self {
            send_command: command_session_new,
        })
    }

    pub async fn session_new(
        &self,
    ) -> crate::result::Result<impl Future<Output = crate::result::Result<WebDriverSession>>> {
        let test = self
            .send_command(
                crate::session::new::Command {
                    params: session::new::Parameters {
                        capabilities: session::new::CapabilitiesRequest {},
                    },
                },
                SendCommand::SessionNew,
            )
            .await?;
        Ok(async {
            let result: session::new::Result = test.await?;
            Ok(WebDriverSession {
                session_id: result.session_id,
                driver: self.clone(),
            })
        })
    }

    pub(crate) async fn send_command<C, R>(
        &self,
        command: C,
        send_command_constructor: impl FnOnce(C, oneshot::Sender<oneshot::Receiver<R>>) -> SendCommand,
    ) -> crate::result::Result<impl Future<Output = crate::result::Result<R>>> {
        let (tx, rx) = oneshot::channel();
        self.send_command
            .send(send_command_constructor(command, tx))
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
