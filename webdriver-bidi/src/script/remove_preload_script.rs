//! <https://w3c.github.io/webdriver-bidi/#command-script-removePreloadScript>

use serde::{Deserialize, Serialize};

use super::{BrowsingContext, ChannelValue, PreloadScript};
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-script-removePreloadScript>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "script.removePreloadScript")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-removePreloadScript>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub script: PreloadScript,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-removePreloadScript>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
