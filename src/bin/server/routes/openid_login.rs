use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use oauth2::{PkceCodeChallenge, Scope};
use openidconnect::core::CoreAuthenticationFlow;
use openidconnect::Nonce;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::error::AppError;
use crate::openid::get_openid_client;
use crate::{CsrfSafeForm, CsrfToken, ExtractSession};

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
    ExtractSession {
        extractor: _form,
        session,
    }: ExtractSession<CsrfSafeForm<OpenIdLoginPayload>>,
) -> Result<impl IntoResponse, AppError> {
    let client = get_openid_client().await?;

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, _csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            openidconnect::CsrfToken::new_random,
            Nonce::new_random,
        )
        // Set the desired scopes.
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("write".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    let mut session = session.lock().await;

    session.set_openid_pkce_verifier(&pkce_verifier);
    session.set_openid_nonce(&nonce);

    drop(session);

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browse to: {auth_url}");

    Ok(Redirect::to("/list").into_response())
}
