use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::SubscriptionRequest;
use crate::browsing_context::BrowsingContext;

/// <https://w3c.github.io/webdriver-bidi/#command-session-subscribe>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "session.subscribe")]
pub struct CommandType {
    pub params: SubscriptionRequest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {}
