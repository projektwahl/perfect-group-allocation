//! <https://w3c.github.io/webdriver-bidi/#protocol-definition>

use serde::{Deserialize, Serialize};

/// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command<T> {
    pub id: u64,
    #[serde(flatten)]
    pub data: T,
    #[serde(flatten)]
    pub extensible: Extensible,
}

/// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Extensible(pub serde_json::Map<String, serde_json::Value>);
