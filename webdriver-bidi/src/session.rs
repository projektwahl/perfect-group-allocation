use serde::{Deserialize, Serialize};

pub mod end;
pub mod new;

/// <https://w3c.github.io/webdriver-bidi/#module-session-definition>
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Command {
    /// https://w3c.github.io/webdriver-bidi/#command-session-new
    SessionNew(new::CommandType),
    /// https://w3c.github.io/webdriver-bidi/#command-session-end
    SessionEnd(end::CommandType),
}
