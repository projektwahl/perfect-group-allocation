//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-traverseHistory>

use serde::{Deserialize, Serialize};

use super::BrowsingContext;
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-traverseHistory>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.traverseHistory")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-traverseHistory>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub context: BrowsingContext,
    pub delta: i64,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-traverseHistory>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {}
