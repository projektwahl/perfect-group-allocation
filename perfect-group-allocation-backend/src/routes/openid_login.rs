use perfect_group_allocation_database::DatabaseConnection;
use perfect_group_allocation_openidconnect::begin_authentication;
use serde::Deserialize;

use crate::error::AppError;
use crate::session::Session;
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
    DatabaseConnection(_db): DatabaseConnection,
    mut session: Session,
    //_form: CsrfSafeForm<OpenIdLoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    // TODO FIXME check csrf token?

    let (auth_url, session) = begin_authentication().await?;

    session.set_openidconnect(&session)?;

    Ok(Redirect::to(auth_url.as_str()).into_response())
}
