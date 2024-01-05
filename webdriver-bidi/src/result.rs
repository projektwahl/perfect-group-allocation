use std::convert::Infallible;

use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

#[derive(Error, Debug)]
pub enum Error {
    #[error("WebSocket connection failure {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Failed to create temporary directory {0}")]
    TmpDirCreateError(std::io::Error),
    #[error("Failed to spawn browser {0}")]
    SpawnBrowserError(std::io::Error),
    #[error("Failed to read browser's stderr {0}")]
    ReadBrowserStderr(std::io::Error),
    #[error("Failed to run browser {0}")]
    FailedToRunBrowser(std::io::Error),
    #[error("Internal tokio oneshot channel receive error {0}")]
    TokioOneShotReceive(tokio::sync::oneshot::error::RecvError),
    #[error("failed to detect WebDriver BiDi port")]
    PortDetectError(std::num::ParseIntError),
    #[error("failed to parse received message {0}")]
    ParseReceived(serde_json::Error),
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
