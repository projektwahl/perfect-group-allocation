//! <https://w3c.github.io/webdriver-bidi/#command-script-getRealms>

use serde::{Deserialize, Serialize};

use super::{BrowsingContext, RealmInfo, RealmType};

/// <https://w3c.github.io/webdriver-bidi/#command-script-getRealms>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "script.getRealms")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-getRealms>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub context: Option<BrowsingContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub r#type: Option<RealmType>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-getRealms>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    realms: Vec<RealmInfo>,
}
