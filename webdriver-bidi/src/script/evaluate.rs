//! <https://w3c.github.io/webdriver-bidi/#command-script-evaluate>

use serde::{Deserialize, Serialize};

use super::{EvaluateResult, ResultOwnership, SerializationOptions, Target};

/// <https://w3c.github.io/webdriver-bidi/#command-script-evaluate>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "script.evaluate")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-evaluate>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub expression: String,
    pub target: Target,
    pub await_promise: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub result_ownership: Option<ResultOwnership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub serialization_options: Option<SerializationOptions>,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub user_activation: Option<bool>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-script-evaluate>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EvaluateResult);

#[cfg(test)]
mod tests {
    use crate::script::{EvaluateResult, RemoteValue};

    #[test]
    fn it_works() {
        let string = r#"{"type":"node","sharedId":"5638c701-0013-4a51-9157-c648b99e48e0","value":{"nodeType":1,"localName":"button","namespaceURI":"http://www.w3.org/1999/xhtml","childNodeCount":1,"children":[{"type":"node","sharedId":"5f9fd48a-cc26-4313-a4ad-a440bab48d13","value":{"nodeType":3,"nodeValue":"Login","childNodeCount":0}}],"attributes":{"id":"login-button","class":"submit-link","type":"submit"},"shadowRoot":null}}"#;
        let jd = &mut serde_json::Deserializer::from_str(string);
        let result: RemoteValue = serde_path_to_error::deserialize(jd).unwrap();
    }
}
