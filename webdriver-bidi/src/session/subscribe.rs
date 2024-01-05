use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use super::SubscriptionRequest;
use crate::CommandResultPair;

/// <https://w3c.github.io/webdriver-bidi/#command-session-subscribe>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "session.subscribe")]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub params: SubscriptionRequest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {}

impl CommandResultPair<Command, Result> for () {
    fn create_respond_command(
        input: oneshot::Sender<Result>,
    ) -> crate::webdriver_handler::RespondCommand {
        todo!()
    }
}
