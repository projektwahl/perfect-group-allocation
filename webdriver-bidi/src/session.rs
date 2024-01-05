use serde::{Deserialize, Serialize};

use crate::browsing_context::BrowsingContext;

pub mod end;
pub mod new;
pub mod subscribe;

/// <https://w3c.github.io/webdriver-bidi/#module-session-definition>
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Command {
    /// https://w3c.github.io/webdriver-bidi/#command-session-new
    SessionNew(new::CommandType),
    /// https://w3c.github.io/webdriver-bidi/#command-session-end
    SessionEnd(end::CommandType),
    /// https://w3c.github.io/webdriver-bidi/#command-session-subscribe
    Subscribe(subscribe::CommandType),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    pub events: Vec<String>,
    pub contexts: Vec<BrowsingContext>,
}
