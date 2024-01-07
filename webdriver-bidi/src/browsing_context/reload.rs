//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-reload>

use serde::{Deserialize, Serialize};

use super::{navigate, BrowsingContext, ReadinessState};

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-reload>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.reload")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-reload>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub context: BrowsingContext,
    pub ignore_cache: Option<bool>,
    pub wait: Option<ReadinessState>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-reload>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub navigate::Result);
