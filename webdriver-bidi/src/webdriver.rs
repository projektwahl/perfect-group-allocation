use std::process::Stdio;

use futures::Future;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{error, trace};

use crate::generated::SendCommand;
use crate::webdriver_handler::WebDriverHandler;

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
    pub async fn new(browser: Browser) -> Result<Self, crate::error::Error> {
        // It's important to not drop this while the browser is running so the directory is not deleted
        let tmp_dir = tempdir().map_err(crate::error::Error::TmpDirCreate)?;

        let port = match browser {
            Browser::Firefox => {
                // oh this path is the culprit?

                let mut child = tokio::process::Command::new("firefox")
                    .kill_on_drop(true)
                    .args([
                        "--profile",
                        &tmp_dir.path().to_string_lossy(),
                        //"--headless",
                        "--remote-debugging-port",
                        "0",
                    ])
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(crate::error::Error::SpawnBrowser)?;

                let stderr = child.stderr.take().unwrap();

                let mut reader = BufReader::new(stderr).lines();

                // Ensure the child process is spawned in the runtime so it can
                // make progress on its own while we await for any output.
                tokio::spawn(async move {
                    let status = child
                        .wait()
                        .await
                        .map_err(crate::error::Error::FailedToRunBrowser)?;

                    error!("child status was: {status}");

                    Ok::<(), crate::error::Error>(())
                });

                let mut port = None;
                while let Some(line) = reader
                    .next_line()
                    .await
                    .map_err(crate::error::Error::ReadBrowserStderr)?
                {
                    trace!(target: "firefox", "{line}");
                    if let Some(p) =
                        line.strip_prefix("WebDriver BiDi listening on ws://127.0.0.1:")
                    {
                        port = Some(p.parse::<u16>().map_err(crate::error::Error::PortDetect)?);
                        break;
                    }
                }

                let Some(port) = port else {
                    return Err(crate::error::Error::PortNotFound)?;
                };

                tokio::spawn(async move {
                    while let Some(line) = reader.next_line().await? {
                        trace!(target: "firefox", "{line}");
                    }
                    Ok::<(), std::io::Error>(())
                });

                port
            }
            Browser::Chromium => {
                // I think my wifi is just broken (surprise).
                // https://netlog-viewer.appspot.com/#import
                // chrome://net-export/
                let mut child = tokio::process::Command::new("chromedriver")
                    .arg("--enable-chrome-logs")
                    //.arg("--log-level=ALL")
                    .kill_on_drop(true)
                    .stdout(Stdio::piped())
                    .spawn()
                    .map_err(crate::error::Error::SpawnBrowser)?;

                let stderr = child.stdout.take().unwrap();

                let mut reader = BufReader::new(stderr).lines();

                // Ensure the child process is spawned in the runtime so it can
                // make progress on its own while we await for any output.
                tokio::spawn(async move {
                    let status = child
                        .wait()
                        .await
                        .map_err(crate::error::Error::FailedToRunBrowser)?;

                    error!("child status was: {status}");

                    Ok::<(), crate::error::Error>(())
                });

                while let Some(line) = reader
                    .next_line()
                    .await
                    .map_err(crate::error::Error::ReadBrowserStderr)?
                {
                    trace!("{line}");
                    if line == "ChromeDriver was started successfully." {
                        break;
                    }
                }

                tokio::spawn(async move {
                    while let Some(line) = reader.next_line().await? {
                        trace!("{line}");
                    }
                    Ok::<(), std::io::Error>(())
                });
                9515
            }
        };

        println!("GOT A PORT");

        let (stream, _response) =
            tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/session"))
                .await
                .map_err(crate::error::Error::WebSocket)?;

        let (command_sender, command_receiver) = mpsc::unbounded_channel();

        // TODO also pass browser process there? or somehow ensure it gets discarded on exit including the profile directroy
        tokio::spawn(WebDriverHandler::handle(tmp_dir, stream, command_receiver));

        Ok(Self {
            send_command: command_sender,
        })
    }

    pub fn send_command<C: Send, R: Send>(
        &self,
        send_command_constructor: impl FnOnce(C, oneshot::Sender<R>) -> SendCommand + Send,
        command: C,
    ) -> impl Future<Output = crate::error::Result<R>> {
        let (tx, rx) = oneshot::channel();

        self.send_command
            .send(send_command_constructor(command, tx))
            .unwrap();

        async {
            let result = rx
                .await
                .map_err(|_| crate::error::Error::CommandTaskExited)?;
            Ok(result)
        }
    }

    pub fn request_subscribe<C: Send, R: Send>(
        &self,
        send_command_constructor: impl FnOnce(C, oneshot::Sender<broadcast::Receiver<R>>) -> SendCommand
            + Send,
        command: C,
    ) -> impl Future<Output = crate::error::Result<broadcast::Receiver<R>>> {
        let (tx, rx) = oneshot::channel();

        self.send_command
            .send(send_command_constructor(command, tx))
            .unwrap();

        async {
            let result = rx
                .await
                .map_err(|_| crate::error::Error::CommandTaskExited)?;
            Ok(result)
        }
    }
}
