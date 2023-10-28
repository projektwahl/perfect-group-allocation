use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct WithCsrfToken<'a, T> {
    pub csrf_token: &'a str,
    #[serde(flatten)]
    pub inner: T,
}
