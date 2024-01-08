//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-setViewport>

use serde::{Deserialize, Serialize};

use super::BrowsingContext;
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-setViewport>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.setViewport")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-setViewport>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub context: BrowsingContext,
    // TODO FIXME null
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub viewport: Option<Viewport>,
    // TODO FIXME null
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub device_pixel_ratio: Option<f64>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-setViewport>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Viewport {
    width: u64,
    height: u64,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-setViewport>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
