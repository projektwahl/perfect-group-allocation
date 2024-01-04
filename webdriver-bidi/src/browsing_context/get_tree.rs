use serde::{Deserialize, Serialize};

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.getTree")]
pub struct CommandType {
    params: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {
    max_depth: u64,
    /// browsing context
    root: String,
}
