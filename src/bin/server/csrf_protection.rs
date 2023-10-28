use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct WithCsrfToken<'tkn, T> {
    pub csrf_token: &'tkn str,
    #[serde(flatten)]
    pub inner: T,
}
