use serde::{Deserialize, Serialize};

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
    /// browsing context
    pub root: Option<String>,
}
