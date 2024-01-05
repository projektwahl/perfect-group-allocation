use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::CommandResultPair;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "session.end")]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub params: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {}

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
