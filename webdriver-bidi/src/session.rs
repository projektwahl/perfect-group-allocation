use serde::{Deserialize, Serialize};

use crate::browsing_context::BrowsingContext;

pub mod end;
pub mod new;
pub mod subscribe;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SubscriptionRequest {
    pub events: Vec<String>,
    pub contexts: Vec<BrowsingContext>,
}
