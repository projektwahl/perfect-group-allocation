use serde::{Deserialize, Serialize};

pub type Event = EntryAdded;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[serde(rename = "log.entryAdded")]
pub struct EntryAdded {
    pub params: Entry,
}

/// <https://w3c.github.io/webdriver-bidi/#types-log-logentry>
#[derive(Debug, Serialize, Deserialize)]
pub enum Entry {
    Log(GenericLogEntry),
    Console(ConsoleLogEntry),
    Javascript(JavascriptLogEntry),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseLogEntry {
    level: String,
    source: String,
    text: Option<String>,
    timestamp: u64,
    stack_trace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericLogEntry {
    #[serde(flatten)]
    base: BaseLogEntry,
    r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleLogEntry {
    #[serde(flatten)]
    base: BaseLogEntry,
    method: String,
    args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavascriptLogEntry {
    #[serde(flatten)]
    base: BaseLogEntry,
}
