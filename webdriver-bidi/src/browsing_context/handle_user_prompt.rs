//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-handleUserPrompt>
use serde::{Deserialize, Serialize};

use super::BrowsingContext;
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-handleUserPrompt>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.handleUserPrompt")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-handleUserPrompt>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub context: BrowsingContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub accept: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub user_text: Option<String>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-handleUserPrompt>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
