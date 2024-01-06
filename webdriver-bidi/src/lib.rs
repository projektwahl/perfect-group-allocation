#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
pub mod browsing_context;
pub mod log;
pub mod result;
pub mod script;
pub mod session;
pub mod webdriver;
pub mod webdriver_handler;
pub mod webdriver_session;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use webdriver_handler::EventData;

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebDriverBiDiLocalEndMessage<ResultData> {
    #[serde(rename = "success")]
    CommandResponse(WebDriverBiDiLocalEndCommandResponse<ResultData>),
    #[serde(rename = "error")]
    ErrorResponse(WebDriverBiDiLocalEndMessageErrorResponse),
    #[serde(rename = "event")]
    Event(EventData),
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiLocalEndCommandResponse<ResultData> {
    id: u64,
    result: ResultData,
    //#[serde(flatten)]
    //extensible: Value,
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
pub struct WebDriverBiDiRemoteEndCommand<T> {
    id: u64,
    #[serde(flatten)]
    command_data: T,
    //#[serde(flatten)]
    //extensible: Value,
}
