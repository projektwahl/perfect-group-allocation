use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::TypedHeader;
use oauth2::PkceCodeChallenge;
use openidconnect::core::CoreAuthenticationFlow;
use openidconnect::Nonce;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::error::to_error_result;
use crate::openid::get_openid_client;
use crate::session::Session;
use crate::{CsrfSafeForm, CsrfToken, XRequestId};

#[derive(Deserialize)]
pub struct OpenIdLoginPayload {
    csrf_token: String,
}

impl CsrfToken for OpenIdLoginPayload {
    fn csrf_token(&self) -> String {
        self.csrf_token.clone()
    }
}

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn openid_login(
    State(_db): State<DatabaseConnection>,
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    mut session: Session,
    _form: CsrfSafeForm<OpenIdLoginPayload>,
) -> Result<(Session, impl IntoResponse), (Session, impl IntoResponse)> {
    let result = async {
        let client = get_openid_client().await?;

        // Generate a PKCE challenge.
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the full authorization URL.
        let (auth_url, csrf_token, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                openidconnect::CsrfToken::new_random,
                Nonce::new_random,
            )
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();

        session.set_openidconnect(&(&pkce_verifier, &nonce, &csrf_token))?;

        Ok(Redirect::to(auth_url.as_str()).into_response())
    };
    match result.await {
        Ok(ok) => Ok((session, ok)),
        Err(app_error) => Err(to_error_result(session, request_id, app_error).await),
    }
}
