use serde::{Deserialize, Serialize};

pub type Event = EntryAdded;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "log.entryAdded")]
pub struct EntryAdded {
    pub params: Entry,
}

/// <https://w3c.github.io/webdriver-bidi/#types-log-logentry>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    level: Level,
    // TODO FIXME this contains the browsing context or realm
    source: Source,
    text: Option<String>,
    timestamp: u64,
    stack_trace: StackTrace,
    #[serde(flatten)]
    inner: InnerLogEntry,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum InnerLogEntry {
    Console(ConsoleLogEntry),
    Javascript(JavascriptLogEntry),
    #[serde(untagged)]
    Log(GenericLogEntry),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericLogEntry {
    r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsoleLogEntry {
    method: String,
    args: Vec<RemoteValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavascriptLogEntry {}
