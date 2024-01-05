use serde::{Deserialize, Serialize};

pub mod get_tree;

/// <https://w3c.github.io/webdriver-bidi/#module-browsingContext-definition>
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Command {
    /// https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree
    GetTree(get_tree::CommandType),
}
