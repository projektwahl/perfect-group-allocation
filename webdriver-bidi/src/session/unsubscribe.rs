//! <https://w3c.github.io/webdriver-bidi/#command-session-unsubscribe>
use serde::{Deserialize, Serialize};

use super::SubscriptionRequest;
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-session-unsubscribe>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "session.unsubscribe")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: SubscriptionRequest,
}

/// <https://w3c.github.io/webdriver-bidi/#command-session-unsubscribe>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
