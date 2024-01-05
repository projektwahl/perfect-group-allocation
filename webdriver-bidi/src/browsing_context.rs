use serde::{Deserialize, Serialize};

pub mod get_tree;

/// <https://w3c.github.io/webdriver-bidi/#module-browsingContext-definition>
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Command {
    /// https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree
    GetTree(get_tree::CommandType),
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Info>
#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    context: BrowsingContext,
    url: String,
    #[serde(default)]
    children: Vec<Info>,
    parent: Option<BrowsingContext>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Browsingcontext>
#[derive(Debug, Serialize, Deserialize)]
pub struct BrowsingContext(String);
