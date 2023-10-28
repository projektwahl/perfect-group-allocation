use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::TypedHeader;
use futures_util::TryFutureExt;
use handlebars::Handlebars;
use oauth2::PkceCodeChallenge;
use openidconnect::core::CoreAuthenticationFlow;
use openidconnect::Nonce;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::error::AppErrorWithMetadata;
use crate::openid::get_openid_client;
use crate::{CsrfSafeForm, CsrfToken, ExtractSession, XRequestId};

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
    State(handlebars): State<Arc<Handlebars<'static>>>,
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    ExtractSession {
        extractor: _form,
        session,
    }: ExtractSession<CsrfSafeForm<OpenIdLoginPayload>>,
) -> Result<impl IntoResponse, AppErrorWithMetadata> {
    let mut session = session.lock().await;
    let expected_csrf_token = session.session_id();
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

        session.set_openid_pkce_verifier(&pkce_verifier);
        session.set_openid_nonce(&nonce);
        session.set_openid_csrf_token(&csrf_token);

        drop(session);

        Ok(Redirect::to(auth_url.as_str()).into_response())
    };
    result
        .or_else(|app_error| async {
            // TODO FIXME store request id type-safe in body/session
            Err(AppErrorWithMetadata {
                csrf_token: expected_csrf_token.clone(),
                request_id,
                handlebars,
                app_error,
            })
        })
        .await
}
