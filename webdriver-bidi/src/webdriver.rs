use std::process::Stdio;

use futures::Future;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::generated::SendCommand;
use crate::session::{self, CapabilitiesRequest};
use crate::webdriver_handler::WebDriverHandler;
use crate::webdriver_session::WebDriverSession;

/// <https://w3c.github.io/webdriver-bidi>
#[derive(Debug, Clone)]
pub struct WebDriver {
    /// send a command
    send_command: mpsc::UnboundedSender<SendCommand>,
}

pub enum Browser {
    Firefox,
    Chromium,
}

impl WebDriver {
    /// Creates a new [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) connection.
    /// ## Errors
    /// Returns an error if the `WebSocket` connection fails.
    pub async fn new(browser: Browser) -> Result<Self, crate::result::Error> {
        let port = match browser {
            Browser::Firefox => {
                let tmp_dir = tempdir().map_err(crate::result::ErrorInner::TmpDirCreate)?;

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
                    .map_err(crate::result::ErrorInner::SpawnBrowser)?;

                let stderr = child.stderr.take().unwrap();

                let mut reader = BufReader::new(stderr).lines();

                // Ensure the child process is spawned in the runtime so it can
                // make progress on its own while we await for any output.
                tokio::spawn(async move {
                    let status = child
                        .wait()
                        .await
                        .map_err(crate::result::ErrorInner::FailedToRunBrowser)?;

                    println!("child status was: {status}");

                    Ok::<(), crate::result::Error>(())
                });

                let mut port = None;
                while let Some(line) = reader
                    .next_line()
                    .await
                    .map_err(crate::result::ErrorInner::ReadBrowserStderr)?
                {
                    eprintln!("{line}");
                    if let Some(p) =
                        line.strip_prefix("WebDriver BiDi listening on ws://127.0.0.1:")
                    {
                        port = Some(
                            p.parse::<u16>()
                                .map_err(crate::result::ErrorInner::PortDetect)?,
                        );
                        break;
                    }
                }

                let Some(port) = port else {
                    return Err(crate::result::ErrorInner::PortNotFound)?;
                };

                tokio::spawn(async move {
                    while let Some(line) = reader.next_line().await? {
                        eprintln!("{line}");
                    }
                    Ok::<(), std::io::Error>(())
                });

                port
            }
            Browser::Chromium => {
                let mut child = tokio::process::Command::new("chromedriver")
                    .kill_on_drop(true)
                    .stdout(Stdio::piped())
                    .spawn()
                    .map_err(crate::result::ErrorInner::SpawnBrowser)?;

                let stderr = child.stdout.take().unwrap();

                let mut reader = BufReader::new(stderr).lines();

                // Ensure the child process is spawned in the runtime so it can
                // make progress on its own while we await for any output.
                tokio::spawn(async move {
                    let status = child
                        .wait()
                        .await
                        .map_err(crate::result::ErrorInner::FailedToRunBrowser)?;

                    println!("child status was: {status}");

                    Ok::<(), crate::result::Error>(())
                });

                while let Some(line) = reader
                    .next_line()
                    .await
                    .map_err(crate::result::ErrorInner::ReadBrowserStderr)?
                {
                    eprintln!("line: {line}");
                    if line == "ChromeDriver was started successfully." {
                        break;
                    }
                }

                tokio::spawn(async move {
                    while let Some(line) = reader.next_line().await? {
                        eprintln!("{line}");
                    }
                    Ok::<(), std::io::Error>(())
                });
                9515
            }
        };

        let (stream, _response) =
            tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/session"))
                .await
                .map_err(crate::result::ErrorInner::WebSocket)?;

        let (command_sender, command_receiver) = mpsc::unbounded_channel();

        tokio::spawn(WebDriverHandler::handle(stream, command_receiver));

        Ok(Self {
            send_command: command_sender,
        })
    }

    pub fn session_new(&self) -> impl Future<Output = crate::result::Result<WebDriverSession>> {
        let test = self.send_command(
            crate::session::new::Command {
                params: session::new::Parameters {
                    capabilities: CapabilitiesRequest {
                        always_match: None,
                        first_match: None,
                    },
                },
            },
            SendCommand::SessionNew,
        );
        async {
            let result: session::new::Result = test.await?;
            Ok(WebDriverSession {
                session_id: result.session_id,
                driver: self.clone(),
            })
        }
    }

    pub fn send_command<C: Send, R: Send>(
        &self,
        command: C,
        send_command_constructor: impl FnOnce(C, oneshot::Sender<R>) -> SendCommand + Send,
    ) -> impl Future<Output = crate::result::Result<R>> {
        let (tx, rx) = oneshot::channel();

        self.send_command
            .send(send_command_constructor(command, tx))
            .unwrap();

        async {
            let result = rx
                .await
                .map_err(|_| crate::result::ErrorInner::CommandTaskExited)?;
            Ok(result)
        }
    }

    pub fn request_subscribe<C: Send, R: Send>(
        &self,
        command: C,
        send_command_constructor: impl FnOnce(C, oneshot::Sender<broadcast::Receiver<R>>) -> SendCommand
        + Send,
    ) -> impl Future<Output = crate::result::Result<broadcast::Receiver<R>>> {
        let (tx, rx) = oneshot::channel();

        self.send_command
            .send(send_command_constructor(command, tx))
            .unwrap();

        async {
            let result = rx
                .await
                .map_err(|_| crate::result::ErrorInner::CommandTaskExited)?;
            Ok(result)
        }
    }
}
