use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use super::{BrowsingContext, Navigation, ReadinessState};
use crate::CommandResultPair;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.navigate")]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub params: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub context: BrowsingContext,
    pub url: String,
    pub wait: ReadinessState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub navigation: Option<Navigation>,
    pub url: String,
}

impl CommandResultPair<Command, Result> for () {
    fn create_respond_command(
        input: oneshot::Sender<Result>,
    ) -> crate::webdriver_handler::RespondCommand {
        todo!()
    }
}
