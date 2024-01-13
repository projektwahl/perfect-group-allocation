//! <https://w3c.github.io/webdriver-bidi/#command-script-addPreloadScript>

use serde::{Deserialize, Serialize};

use super::{BrowsingContext, ChannelValue, PreloadScript};

/// <https://w3c.github.io/webdriver-bidi/#command-script-addPreloadScript>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "script.addPreloadScript")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-addPreloadScript>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub function_declaration: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub arguments: Option<Vec<ChannelValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub sandbox: Option<String>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-addPreloadScript>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    script: PreloadScript,
}
