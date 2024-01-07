//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-activate>

use serde::{Deserialize, Serialize};

use super::BrowsingContext;
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-activate>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.activate")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-activate>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub context: BrowsingContext,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-activate>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
