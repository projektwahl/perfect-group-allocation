//! <https://w3c.github.io/webdriver-bidi/#module-input>

pub mod perform_actions;

use serde::{Deserialize, Serialize};

use crate::script::SharedReference;

/// <https://w3c.github.io/webdriver-bidi/#module-input>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "element")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ElementOrigin {
    element: SharedReference,
}
