//! <https://w3c.github.io/webdriver-bidi/#protocol>

use serde::{Deserialize, Serialize};

/// <https://w3c.github.io/webdriver-bidi/#protocol>
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command<T> {
    pub id: u64,
    #[serde(flatten)]
    pub data: T,
    #[serde(flatten)]
    pub extensible: Extensible,
}

pub use crate::generated::CommandData;

/// <https://w3c.github.io/webdriver-bidi/#protocol>
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EmptyParams(pub Extensible);

// https://w3c.github.io/webdriver-bidi/#protocol
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Message<ResultData> {
    #[serde(rename = "success")]
    CommandResponse(CommandResponse<ResultData>),
    #[serde(rename = "error")]
    ErrorResponse(ErrorResponse),
    #[serde(rename = "event")]
    Event(EventData),
}

// https://w3c.github.io/webdriver-bidi/#protocol
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CommandResponse<ResultData> {
    #[serde(deserialize_with = "deserialize_broken_chromium_id")]
    pub id: u64,
    pub result: ResultData,
    //#[serde(flatten)]
    //extensible: Value,
}

fn deserialize_broken_chromium_id<'de, D>(deserializer: D) -> core::result::Result<u64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct JsonStringVisitor;

    impl<'de> serde::de::Visitor<'de> for JsonStringVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing json data")
        }

        fn visit_u64<E>(self, v: u64) -> core::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }

        fn visit_f64<E>(self, v: f64) -> core::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok(v as u64)
        }
    }

    // use our visitor to deserialize an `ActualValue`
    deserializer.deserialize_any(JsonStringVisitor)
}

// https://w3c.github.io/webdriver-bidi/#protocol
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ErrorResponse {
    pub id: Option<u64>,
    pub error: String,
    pub message: String,
    pub stacktrace: Option<String>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

pub use crate::generated::ResultData;

/// <https://w3c.github.io/webdriver-bidi/#protocol>
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EmptyResult(pub Extensible);

// https://w3c.github.io/webdriver-bidi/#protocol
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename = "event")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Event {
    #[serde(flatten)]
    pub data: EventData,
    #[serde(flatten)]
    pub extensible: Extensible,
}

pub use crate::generated::EventData;

/// <https://w3c.github.io/webdriver-bidi/#protocol>
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Extensible(pub serde_json::Map<String, serde_json::Value>);
