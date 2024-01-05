#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
pub mod browsing_context;
pub mod session;
pub mod webdriver;
pub mod webdriver_session;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebDriverBiDiLocalEndMessage<ResultData> {
    #[serde(rename = "success")]
    CommandResponse(WebDriverBiDiLocalEndCommandResponse<ResultData>),
    #[serde(rename = "error")]
    ErrorResponse(WebDriverBiDiLocalEndMessageErrorResponse),
    #[serde(rename = "event")]
    Event(WebDriverBiDiLocalEndEvent),
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiLocalEndCommandResponse<ResultData> {
    id: u64,
    result: ResultData,
    // Extensible
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiLocalEndEvent {
    id: u64,
    // TODO EventData
    // Extensible
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiLocalEndMessageErrorResponse {
    id: Option<u64>,
    error: String,
    message: String,
    stacktrace: Option<String>,
    #[serde(flatten)]
    extensible: Value,
}

/// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiRemoteEndCommand {
    id: u64,
    #[serde(flatten)]
    command_data: WebDriverBiDiRemoteEndCommandData,
    #[serde(flatten)]
    extensible: Value,
}

/// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebDriverBiDiRemoteEndCommandData {
    SessionCommand(session::Command),
    BrowsingContext(browsing_context::Command),
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn it_works() -> Result<(), tokio_tungstenite::tungstenite::Error> {
        Ok(())
    }
}
