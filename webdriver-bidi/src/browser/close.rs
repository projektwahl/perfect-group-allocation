//! <https://w3c.github.io/webdriver-bidi/#command-browser-close>

use serde::{Deserialize, Serialize};

use crate::protocol::{EmptyParams, EmptyResult};

/// <https://w3c.github.io/webdriver-bidi/#command-browser-close>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browser.close")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: EmptyParams,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browser-close>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
