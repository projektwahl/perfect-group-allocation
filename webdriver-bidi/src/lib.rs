#![feature(error_generic_member_access)]
#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

//! Client for the [WebDriver BiDi protocol](https://w3c.github.io/webdriver-bidi/).
//!
//! <div class="warning">This crate is in a very early stage of development! Expect frequent breaking changes.</div>
//!
//! ## Implementation Notes
//!
//! The types in <https://w3c.github.io/webdriver-bidi/> are converted with the following in mind:
//!
//! They are converted as much as possible as-is to make it easy to update them.
//!
//! They are converted with performance and ease-of-use in mind.
//! For example tagged unions are converted in a way that only one parsing attempt is needed (so no untagged).
//!
//! ### Type conversion rules
//! | Spec type    | Rust type                    |
//! |--------------|------------------------------|
//! | `js-uint`    | [`u64`]                      |
//! | `js-int`     | [`i64`]                      |
//! | `Extensible` | [`serde_json::value::Value`] |
//!
//! ### Serde rules
//! All types will (at some point) be annotated with:
//! ```
//! #[serde(rename_all = "camelCase")]
//! #[serde(deny_unknown_fields)]
//! ```
//!
//! To specify a fallback type on a tagged enum **variant**, use:
//! ```
//! #[serde(untagged)]
//! ```
//!
//! All fields in the spec starting with `? ` are optional and are represented as `Option<T>` and annotated with:
//! ```rust
//! #[serde(skip_serializing_if = "Option::is_none")]
//! #[serde(default)]
//! ```
//!
//! Otherwise `Type / null` is represented with an `Option<Type>`.
//!
//! Types combined in the spec with `( A // B // ... )` are represented as enum though usually as a tagged enum for performance.

pub mod browsing_context;
pub mod log;
pub mod result;
pub mod script;
pub mod session;
pub mod webdriver;
pub mod webdriver_handler;
pub mod webdriver_session;

use browsing_context::BrowsingContext;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use webdriver_handler::EventData;

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebDriverBiDiLocalEndMessage<ResultData> {
    #[serde(rename = "success")]
    CommandResponse(WebDriverBiDiLocalEndCommandResponse<ResultData>),
    #[serde(rename = "error")]
    ErrorResponse(WebDriverBiDiLocalEndMessageErrorResponse),
    #[serde(rename = "event")]
    Event(EventData),
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiLocalEndCommandResponse<ResultData> {
    #[serde(deserialize_with = "deserialize_broken_chromium_id")]
    id: u64,
    result: ResultData,
    //#[serde(flatten)]
    //extensible: Value,
}

fn deserialize_broken_chromium_id<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    // define a visitor that deserializes
    // `ActualData` encoded as json within a string
    struct JsonStringVisitor;

    impl<'de> serde::de::Visitor<'de> for JsonStringVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing json data")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v as u64)
        }
    }

    // use our visitor to deserialize an `ActualValue`
    deserializer.deserialize_any(JsonStringVisitor)
}

// https://w3c.github.io/webdriver-bidi/#protocol-definition
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiLocalEndMessageErrorResponse {
    id: Option<u64>,
    error: String,
    message: String,
    stacktrace: Option<String>,
    #[serde(flatten)]
    extensible: Value,
}

/// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDriverBiDiRemoteEndCommand<T> {
    id: u64,
    #[serde(flatten)]
    command_data: T,
    //#[serde(flatten)]
    //extensible: Value,
}

pub trait ExtractBrowsingContext {
    fn browsing_context(&self) -> Option<&BrowsingContext>;
}
