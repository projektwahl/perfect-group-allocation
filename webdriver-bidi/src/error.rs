use std::backtrace::Backtrace;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("{inner}\n{backtrace}")]
pub struct Error {
    #[from]
    inner: ErrorInner,
    backtrace: Backtrace,
}

#[derive(Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum ErrorInner {
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
}

pub type Result<T> = core::result::Result<T, Error>;
