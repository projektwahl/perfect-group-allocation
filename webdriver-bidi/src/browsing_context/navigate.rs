//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
use serde::{Deserialize, Serialize};

use super::{BrowsingContext, Navigation, ReadinessState};

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.navigate")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub context: BrowsingContext,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub wait: Option<ReadinessState>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    pub navigation: Option<Navigation>,
    pub url: String,
}
