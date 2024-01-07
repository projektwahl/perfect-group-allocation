//! <https://w3c.github.io/webdriver-bidi/#module-session>

use serde::{Deserialize, Serialize};

use crate::browsing_context::BrowsingContext;
use crate::protocol::Extensible;

pub mod end;
pub mod new;
pub mod status;
pub mod subscribe;
pub mod unsubscribe;

/// <https://w3c.github.io/webdriver-bidi/#type-session-CapabilitiesRequest>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CapabilitiesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub always_match: Option<CapabilityRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub first_match: Option<Vec<CapabilityRequest>>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-session-CapabilityRequest>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub accept_insecure_certs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub browser_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub browser_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub platform_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub proxy: Option<ProxyConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub web_socket_url: Option<bool>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

/// <https://w3c.github.io/webdriver-bidi/#type-session-ProxyConfiguration>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "proxyType")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum ProxyConfiguration {
    #[serde(rename = "autodetect")]
    Autodetect(AutodetectProxyConfiguration),
    #[serde(rename = "direct")]
    Direct(DirectProxyConfiguration),
    #[serde(rename = "manual")]
    Manual(ManualProxyConfiguration),
    #[serde(rename = "pac")]
    PacProxy(PacProxyConfiguration),
    #[serde(rename = "system")]
    System(SystemProxyConfiguration),
}

/// <https://w3c.github.io/webdriver-bidi/#type-session-ProxyConfiguration>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AutodetectProxyConfiguration {
    #[serde(flatten)]
    pub extensible: Extensible,
}

/// <https://w3c.github.io/webdriver-bidi/#type-session-ProxyConfiguration>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DirectProxyConfiguration {
    #[serde(flatten)]
    pub extensible: Extensible,
}

/// <https://w3c.github.io/webdriver-bidi/#type-session-ProxyConfiguration>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ManualProxyConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ftp_proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub http_proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ssl_proxy: Option<String>,
    #[serde(flatten)]
    pub socks_proxy_configuration: Option<SocksProxyConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub no_proxy: Option<Vec<String>>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

/// <https://w3c.github.io/webdriver-bidi/#type-session-ProxyConfiguration>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SocksProxyConfiguration {
    socks_proxy: String,
    socks_version: u8,
}

/// <https://w3c.github.io/webdriver-bidi/#type-session-ProxyConfiguration>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PacProxyConfiguration {
    proxy_autoconfig_url: String,
    #[serde(flatten)]
    pub extensible: Extensible,
}

/// <https://w3c.github.io/webdriver-bidi/#type-session-ProxyConfiguration>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SystemProxyConfiguration {
    #[serde(flatten)]
    pub extensible: Extensible,
}

/// <https://w3c.github.io/webdriver-bidi/#type-session-SubscriptionRequest>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SubscriptionRequest {
    pub events: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub contexts: Option<Vec<BrowsingContext>>,
}
