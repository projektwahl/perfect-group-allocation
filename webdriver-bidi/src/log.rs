use serde::{Deserialize, Serialize};

use crate::script::{RemoteValue, Source, StackTrace};
use crate::ExtractBrowsingContext;

pub type Event = EntryAdded;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "log.entryAdded")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EntryAdded {
    pub params: Entry,
}

impl ExtractBrowsingContext for EntryAdded {
    fn browsing_context(&self) -> Option<&crate::browsing_context::BrowsingContext> {
        self.params.source.context.as_ref()
    }
}

/// <https://w3c.github.io/webdriver-bidi/#types-log-logentry>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

/// <https://w3c.github.io/webdriver-bidi/#types-log-logentry>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Entry {
    level: Level,
    source: Source,
    text: Option<String>,
    timestamp: u64,
    stack_trace: Option<StackTrace>,
    #[serde(flatten)]
    inner: InnerLogEntry,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum InnerLogEntry {
    Console(ConsoleLogEntry),
    Javascript(JavascriptLogEntry),
    #[serde(untagged)]
    Log(GenericLogEntry),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct GenericLogEntry {
    r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ConsoleLogEntry {
    method: String,
    args: Vec<RemoteValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct JavascriptLogEntry {}
