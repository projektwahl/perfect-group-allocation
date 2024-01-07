//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot>

use serde::{Deserialize, Serialize};

use super::BrowsingContext;
use crate::script::SharedReference;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.captureScreenshot")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub context: BrowsingContext,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub origin: Option<Origin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub format: Option<ImageFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub clip: Option<ClipRectangle>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Origin {
    Viewport,
    Document,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ImageFormat {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub quality: Option<f64>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum ClipRectangle {
    Element(ElementClipRectangle),
    Box(BoxClipRectangle),
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BoxClipRectangle {
    element: SharedReference,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ElementClipRectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    data: String,
}
