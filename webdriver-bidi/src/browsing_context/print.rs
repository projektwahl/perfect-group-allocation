//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-print>

use serde::{Deserialize, Serialize};

use super::BrowsingContext;
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-print>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.activate")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-print>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub context: BrowsingContext,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub margin: Option<PrintMarginParameters>,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub orientation: Option<Orientation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub page: Option<PrintPageParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub page_ranges: Option<Vec<PageRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub shrink_to_fit: Option<bool>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-print>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-print>
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum PageRange {
    Number(u64),
    String(String),
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-print>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PrintMarginParameters {
    // TODO FIXME default
    /// must be >= 0
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub bottom: Option<f64>,
    // TODO FIXME default
    /// must be >= 0
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub left: Option<f64>,
    // TODO FIXME default
    /// must be >= 0
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub right: Option<f64>,
    // TODO FIXME default
    /// must be >= 0
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub top: Option<f64>,
}

/// Minimum size is 1pt x 1pt. Conversion follows from
/// https://www.w3.org/TR/css3-values/#absolute-lengths
/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-print>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PrintPageParameters {
    // TODO FIXME default
    /// must be >= 0.0352
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub height: Option<f64>,
    // TODO FIXME default
    /// must be >= 0.0352
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub width: Option<f64>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-print>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
