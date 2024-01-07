//! <https://w3c.github.io/webdriver-bidi/#command-session-new>
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{CapabilitiesRequest, ProxyConfiguration};
use crate::protocol::Extensible;

/// <https://w3c.github.io/webdriver-bidi/#command-session-new>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "session.new")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-session-new>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub capabilities: CapabilitiesRequest,
}

/// <https://w3c.github.io/webdriver-bidi/#command-session-new>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result {
    pub session_id: String,
    pub capabilities: Capabilities,
}

/// <https://w3c.github.io/webdriver-bidi/#command-session-new>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub accept_insecure_certs: bool,
    pub browser_name: String,
    pub browser_version: String,
    pub platform_name: String,
    pub set_window_rect: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    proxy: Option<ProxyConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    web_socket_url: Option<String>,
    #[serde(flatten)]
    pub extensible: Extensible,
}
