use bytes::Bytes;
use headers::Location;
use http::header::LOCATION;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::{BodyExt as _, Empty};
use perfect_group_allocation_database::DatabaseConnection;
use perfect_group_allocation_openidconnect::begin_authentication;
use serde::Deserialize;

use crate::error::AppError;
use crate::session::Session;
use crate::{CsrfToken, ResponseTypedHeaderExt};

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
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = AppError>>, AppError> {
    // TODO FIXME check csrf token?

    let (auth_url, openid_session) = begin_authentication().await?;

    session.set_openidconnect(&openid_session)?;

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(LOCATION, auth_url)
        .body(Empty::new().map_err::<_, AppError>(|err| match err {}))
        .unwrap())
}
