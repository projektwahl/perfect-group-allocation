use std::convert::Infallible;

use bytes::Bytes;

use http::header::LOCATION;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::{Empty};

use perfect_group_allocation_openidconnect::begin_authentication;
use serde::Deserialize;

use crate::error::AppError;
use crate::session::Session;
use crate::{CsrfToken};

#[derive(Deserialize)]
pub struct OpenIdLoginPayload {
    csrf_token: String,
}

impl CsrfToken for OpenIdLoginPayload {
    fn csrf_token(&self) -> String {
        self.csrf_token.clone()
    }
}

pub async fn openid_login(
    mut session: Session,
    //_form: CsrfSafeForm<OpenIdLoginPayload>,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible>>, AppError> {
    // TODO FIXME check csrf token?

    let (auth_url, openid_session) = begin_authentication().await?;

    session.set_openidconnect(&openid_session)?;

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(LOCATION, auth_url)
        .body(Empty::new())
        .unwrap())
}
