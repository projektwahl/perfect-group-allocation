use serde::{Deserialize, Serialize};

use super::{BrowsingContext, Info, NavigationInfo};

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-contextCreated>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.contextCreated")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ContextCreated {
    pub params: Info,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-contextDestroyed>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.contextDestroyed")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ContextDestroyed {
    pub params: Info,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-navigationStarted>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.navigationStarted")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NavigationStarted {
    pub params: NavigationInfo,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-fragmentNavigated>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.fragmentNavigated")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct FragmentNavigated {
    pub params: NavigationInfo,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-domContentLoaded>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.domContentLoaded")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DomContentLoaded {
    pub params: NavigationInfo,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-load>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.load")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Load {
    pub params: NavigationInfo,
}

// TODO FIXME typo in url
/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-downoadWillBegin>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.downloadWillBegin")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DownloadWillBegin {
    pub params: NavigationInfo,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-navigationAborted>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.navigationAborted")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NavigationAborted {
    pub params: NavigationInfo,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-navigationFailed>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.navigationFailed")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NavigationFailed {
    pub params: NavigationInfo,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-userPromptClosed>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.userPromptClosed")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct UserPromptClosed {
    pub params: UserPromptClosedParameters,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-userPromptClosed>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct UserPromptClosedParameters {
    pub context: BrowsingContext,
    pub accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub user_text: Option<String>,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-userPromptOpened>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.userPromptOpened")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct UserPromptOpened {
    pub params: UserPromptOpenedParameters,
}

/// <https://w3c.github.io/webdriver-bidi/#event-browsingContext-userPromptOpened>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct UserPromptOpenedParameters {
    pub context: BrowsingContext,
    pub r#type: UserPromptType,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub default_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum UserPromptType {
    Alert,
    Confirm,
    Prompt,
    Beforeunload,
}
