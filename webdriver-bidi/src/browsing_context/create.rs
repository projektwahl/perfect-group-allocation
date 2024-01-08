//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-create>
use serde::{Deserialize, Serialize};

use super::BrowsingContext;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-create>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.create")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-create>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Type {
    Tab,
    Window,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-create>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub r#type: Type,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub reference_context: Option<BrowsingContext>,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub background: Option<bool>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-create>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    pub context: BrowsingContext,
}
