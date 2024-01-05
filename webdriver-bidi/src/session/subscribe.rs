use serde::{Deserialize, Serialize};


use super::SubscriptionRequest;


/// <https://w3c.github.io/webdriver-bidi/#command-session-subscribe>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "session.subscribe")]
#[serde(rename_all = "camelCase")]
pub struct CommandType {
    pub params: SubscriptionRequest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {}
