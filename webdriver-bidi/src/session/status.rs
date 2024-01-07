//! <https://w3c.github.io/webdriver-bidi/#command-session-status>
use serde::{Deserialize, Serialize};

use crate::protocol::EmptyParams;

/// <https://w3c.github.io/webdriver-bidi/#command-session-status>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "session.status")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: EmptyParams,
}

/// <https://w3c.github.io/webdriver-bidi/#command-session-status>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    ready: bool,
    message: String,
}
