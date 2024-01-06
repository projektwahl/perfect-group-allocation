use std::process::Stdio;

use futures::Future;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::session;
use crate::webdriver_handler::{SendCommand, SendSubscribeGlobalEvent, WebDriverHandler};
use crate::webdriver_session::WebDriverSession;

/// <https://w3c.github.io/webdriver-bidi>
#[derive(Debug, Clone)]
pub struct WebDriver {
    /// send a command
    send_command: mpsc::Sender<SendCommand>,
    // send a subscribe command and receive the subscription channel.
    event_handlers_log: mpsc::Sender<SendSubscribeGlobalEvent>,
}

impl WebDriver {
    /// Creates a new [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) connection.
    /// ## Errors
    /// Returns an error if the `WebSocket` connection fails.
    pub async fn new() -> Result<Self, crate::result::Error> {
        let tmp_dir = tempdir().map_err(crate::result::Error::TmpDirCreate)?;

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
            .map_err(crate::result::Error::SpawnBrowser)?;

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
                port = Some(p.parse::<u16>().map_err(crate::result::Error::PortDetect)?);
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

        let (stream, _response) =
            tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/session")).await?;

        let (command_sender, command_receiver) = mpsc::channel(10);
        let (subscription_sender, subscription_receiver) = mpsc::channel(10);

        tokio::spawn(WebDriverHandler::handle(
            stream,
            command_receiver,
            subscription_receiver,
        ));

        Ok(Self {
            send_command: command_sender,
            event_handlers_log: subscription_sender,
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

    pub(crate) async fn send_command<C: Send, R: Send>(
        &self,
        command: C,
        send_command_constructor: impl FnOnce(C, oneshot::Sender<oneshot::Receiver<R>>) -> SendCommand
        + Send,
    ) -> crate::result::Result<impl Future<Output = crate::result::Result<R>>> {
        let (tx, rx) = oneshot::channel();

        // maybe use an unbounded sender, then we don't need async here
        self.send_command
            .send(send_command_constructor(command, tx))
            .await
            .map_err(|_| crate::result::Error::CommandTaskExited)?;

        // TODO FIXME I think we don't need this intermediate part as send already guarantees order
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
