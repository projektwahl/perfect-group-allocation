use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum Error {
    #[error("WebSocket connection failure {0}")]
    WebSocket(tokio_tungstenite::tungstenite::Error),
    #[error("Failed to create temporary directory {0}")]
    TmpDirCreate(std::io::Error),
    #[error("Failed to spawn browser {0}")]
    SpawnBrowser(std::io::Error),
    #[error("Failed to read browser's stderr {0}")]
    ReadBrowserStderr(std::io::Error),
    #[error("Failed to run browser {0}")]
    FailedToRunBrowser(std::io::Error),
    #[error("Internal tokio oneshot channel receive error {0}")]
    TokioOneShotReceive(tokio::sync::oneshot::error::RecvError),
    #[error("failed to detect WebDriver BiDi port")]
    PortDetect(std::num::ParseIntError),
    #[error("failed to parse received message {0:?}")]
    ParseReceivedWithPath(serde_path_to_error::Error<serde_json::Error>),
    #[error("failed to find WebDriver BiDi port")]
    PortNotFound,
    #[error("the command task exited. this may be because you requested it or because it crashed")]
    CommandTaskExited,
    #[error(
        "a caller that wanted to execute a command has exited. this may be because it panicked."
    )]
    CommandCallerExited,
    #[error("got response without corresponding request for id {0}")]
    ResponseWithoutRequest(u64),
    #[error("failed to find element with css selector {0}")]
    ElementNotFound(String),
}

pub type Result<T> = core::result::Result<T, Error>;
