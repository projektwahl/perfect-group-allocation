//! <https://w3c.github.io/webdriver-bidi/#command-script-evaluate>

use serde::{Deserialize, Serialize};

use super::{
    BrowsingContext, ChannelValue, EvaluateResult, LocalValue, PreloadScript, ResultOwnership,
    SerializationOptions, Target,
};

/// <https://w3c.github.io/webdriver-bidi/#command-script-evaluate>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "script.evaluate")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-evaluate>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub expression: String,
    pub target: Target,
    pub await_promise: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub result_ownership: Option<ResultOwnership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub serialization_options: Option<SerializationOptions>,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub user_activation: Option<bool>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-evaluate>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EvaluateResult);
