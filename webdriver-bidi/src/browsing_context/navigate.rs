use serde::{Deserialize, Serialize};

use super::{BrowsingContext, Navigation, ReadinessState};

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.navigate")]
pub struct CommandType {
    pub params: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {
    pub context: BrowsingContext,
    pub url: String,
    pub wait: ReadinessState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Result {
    navigation: Option<Navigation>,
    pub url: String,
}
