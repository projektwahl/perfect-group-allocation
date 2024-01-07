use serde::{Deserialize, Serialize};

use super::BrowsingContext;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.create")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub r#type: String, // TODO FIXME tab or window
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub reference_context: Option<BrowsingContext>,
    pub background: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    pub context: BrowsingContext,
}
