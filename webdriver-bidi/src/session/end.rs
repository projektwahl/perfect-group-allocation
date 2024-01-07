//! <https://w3c.github.io/webdriver-bidi/#command-session-end>

use serde::{Deserialize, Serialize};

use crate::protocol::{EmptyParams, EmptyResult};

/// <https://w3c.github.io/webdriver-bidi/#command-session-end>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "session.end")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: EmptyParams,
}

/// <https://w3c.github.io/webdriver-bidi/#command-session-end>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
