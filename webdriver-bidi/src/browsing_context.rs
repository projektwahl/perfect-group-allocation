//! <https://w3c.github.io/webdriver-bidi/#module-browsingContext>

use serde::{Deserialize, Serialize};

use crate::ExtractBrowsingContext;

pub mod activate;
pub mod capture_screenshot;
pub mod close;
pub mod create;
pub mod event;
pub mod get_tree;
pub mod handle_user_prompt;
pub mod locate_nodes;
pub mod navigate;
pub mod print;
pub mod reload;
pub mod set_viewport;
pub mod traverse_history;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.load")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Load {
    pub params: NavigationInfo,
}

impl ExtractBrowsingContext for Load {
    fn browsing_context(&self) -> Option<&crate::browsing_context::BrowsingContext> {
        Some(&self.params.context)
    }
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Browsingcontext>
#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BrowsingContext(pub String);

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Info>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct InfoList(pub Vec<Info>);

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Info>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Info {
    pub context: BrowsingContext,
    pub url: String,
    #[serde(default)]
    pub children: Option<InfoList>,
    // TODO FIXME optional or null unclear
    pub parent: Option<BrowsingContext>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Locator>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Locator {
    #[serde(rename = "css")]
    Css(CssLocator),
    #[serde(rename = "innerText")]
    InnerText(InnerTextLocator),
    #[serde(rename = "xpath")]
    XPath(XPathLocator),
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Locator>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CssLocator {
    pub value: String,
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Locator>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct InnerTextLocator {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ignore_case: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub match_type: Option<MatchType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_depth: Option<u64>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Locator>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum MatchType {
    Full,
    Partial,
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Locator>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct XPathLocator {
    pub value: String,
}

/// The `Navigation` type is a unique string identifying an ongoing navigation.
/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-Navigation>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Navigation(pub String);

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-NavigationInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NavigationInfo {
    pub context: BrowsingContext,
    pub navigation: Option<Navigation>,
    pub timestamp: u64,
    pub url: String,
}

/// <https://w3c.github.io/webdriver-bidi/#type-browsingContext-ReadinessState>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum ReadinessState {
    None,
    Interactive,
    Complete,
}
