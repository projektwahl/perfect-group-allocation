//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>

use serde::{Deserialize, Serialize};

use super::{BrowsingContext, InfoList};

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.getTree")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_depth: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub root: Option<BrowsingContext>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    pub contexts: InfoList,
}
