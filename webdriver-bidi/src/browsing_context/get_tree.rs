use serde::{Deserialize, Serialize};

use super::BrowsingContext;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.getTree")]
pub struct CommandType {
    pub params: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {
    pub max_depth: Option<u64>,
    pub root: Option<BrowsingContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Result {
    pub contexts: Vec<super::Info>,
}
