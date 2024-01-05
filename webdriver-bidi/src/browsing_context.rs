use serde::{Deserialize, Serialize};

pub mod get_tree;
pub mod navigate;

/// <https://w3c.github.io/webdriver-bidi/#module-browsingContext-definition>
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Command {
    /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
    GetTree(get_tree::Command),
    /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
    Navigate(navigate::Command),
}

/// <https://w3c.github.io/webdriver-bidi/#module-browsingContext-definition>
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Result {
    /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
    GetTree(get_tree::Result),
    /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
    Navigate(navigate::Result),
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Info>
#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub context: BrowsingContext,
    pub url: String,
    #[serde(default)]
    pub children: Vec<Info>,
    pub parent: Option<BrowsingContext>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Browsingcontext>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowsingContext(pub String);

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-ReadinessState>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReadinessState {
    None,
    Interactive,
    Complete,
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Navigation>
/// The `Navigation` type is a unique string identifying an ongoing navigation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Navigation(pub String);