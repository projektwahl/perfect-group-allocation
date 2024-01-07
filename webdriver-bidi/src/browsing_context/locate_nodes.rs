//! <https://w3c.github.io/webdriver-bidi/#command-browsingContext-locateNodes>
use serde::{Deserialize, Serialize};

use super::{BrowsingContext, Locator};
use crate::protocol::EmptyResult;
use crate::script::{NodeRemoteValue, ResultOwnership, SerializationOptions, SharedReference};

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-locateNodes>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.handleUserPrompt")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-locateNodes>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub context: BrowsingContext,
    pub locator: Locator,
    /// This needs to be >= 1
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_node_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ownership: Option<ResultOwnership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub sandbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub serialization_options: Option<SerializationOptions>,
    /// This is not allowed to be an empty list
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub start_nodes: Option<Vec<SharedReference>>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-locateNodes>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    nodes: Vec<NodeRemoteValue>,
}
