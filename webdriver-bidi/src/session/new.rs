use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::oneshot;

use crate::CommandResultPair;

/// <https://w3c.github.io/webdriver-bidi/#module-session-definition>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "session.new")]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub params: Parameters,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub capabilities: CapabilitiesRequest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesRequest {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub session_id: String,
    pub capabilities: ResultCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultCapabilities {
    pub accept_insecure_certs: bool,
    pub browser_name: String,
    pub browser_version: String,
    pub platform_name: String,
    pub set_window_rect: bool,
    //proxy: Option<session.ProxyConfiguration>,
    //webSocketUrl: Option<text / true>,
    #[serde(flatten)]
    pub extensible: Value,
}

impl CommandResultPair<Command, Result> for () {
    fn create_respond_command(
        input: oneshot::Sender<Result>,
    ) -> crate::webdriver_handler::RespondCommand {
        todo!()
    }
}
