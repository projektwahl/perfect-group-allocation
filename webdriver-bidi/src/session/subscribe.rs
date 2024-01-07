//! <https://w3c.github.io/webdriver-bidi/#command-session-subscribe>
use serde::{Deserialize, Serialize};

use super::SubscriptionRequest;
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-session-subscribe>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "session.subscribe")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: SubscriptionRequest,
}

/// <https://w3c.github.io/webdriver-bidi/#command-session-subscribe>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);
