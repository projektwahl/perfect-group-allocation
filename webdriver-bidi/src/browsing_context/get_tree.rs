use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use super::BrowsingContext;
use crate::CommandResultPair;

/// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "browsingContext.getTree")]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub params: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub max_depth: Option<u64>,
    pub root: Option<BrowsingContext>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub contexts: Vec<super::Info>,
}

impl CommandResultPair<Command, Result> for () {
    fn create_respond_command(
        input: oneshot::Sender<Result>,
    ) -> crate::webdriver_handler::RespondCommand {
        todo!()
    }
}
