use perfect_group_allocation_database::DatabaseConnection;
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
    let client = match OPENID_CLIENT.get().unwrap() {
        Ok(client) => client,
        Err(_error) => return Err(AppError::OpenIdNotConfigured),
    };

    // TODO FIXME check csrf token?

    session.set_openidconnect(&(&pkce_verifier, &nonce, &csrf_token))?;

    Ok(Redirect::to(auth_url.as_str()).into_response())
}
