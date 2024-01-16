use std::convert::Infallible;

use bytes::{Buf, Bytes};
use http::header::LOCATION;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::Empty;
use perfect_group_allocation_config::Config;
use perfect_group_allocation_openidconnect::begin_authentication;
use serde::Deserialize;

use crate::error::AppError;
use crate::session::{ResponseSessionExt as _, Session};
use crate::CsrfToken;

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
    session: Session,
    config: Config,
    //_form: CsrfSafeForm<OpenIdLoginPayload>,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible> + Send + 'static>, AppError>
{
    // TODO FIXME check csrf token?

    let (auth_url, openid_session) = begin_authentication(config).await?;

    let session = session.with_temporary_openidconnect_state(openid_session);

    Ok(Response::builder()
        .with_session(session)
        .status(StatusCode::SEE_OTHER)
        .header(LOCATION, auth_url)
        .body(Empty::new())
        .unwrap())
}
