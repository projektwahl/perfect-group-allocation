//! <https://w3c.github.io/webdriver-bidi/#command-script-callFunction>

use serde::{Deserialize, Serialize};

use super::{EvaluateResult, LocalValue, ResultOwnership, SerializationOptions, Target};

/// <https://w3c.github.io/webdriver-bidi/#command-script-callFunction>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "script.callFunction")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-callFunction>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub function_declaration: String,
    pub await_promise: bool,
    pub target: Target,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub arguments: Option<Vec<LocalValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub result_ownership: Option<ResultOwnership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub serialization_options: Option<SerializationOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub this: Option<LocalValue>,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub user_activation: Option<bool>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-callFunction>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EvaluateResult);
