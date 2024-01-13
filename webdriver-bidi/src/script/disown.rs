//! <https://w3c.github.io/webdriver-bidi/#command-script-disown>

use serde::{Deserialize, Serialize};

use super::{Handle, Target};
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-script-disown>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "script.disown")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-disown>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub handles: Vec<Handle>,
    pub target: Target,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-disown>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
