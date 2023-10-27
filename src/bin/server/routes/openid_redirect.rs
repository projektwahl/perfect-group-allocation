use anyhow::anyhow;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse as OAuth2TokenResponse};
use openidconnect::{AccessTokenHash, TokenResponse as OpenIdTokenResponse};
use sea_orm::DatabaseConnection;

use crate::error::AppError;
use crate::openid::get_openid_client;
use crate::{CreateProjectPayload, CsrfSafeForm, ExtractSession, MyBody, MyState};

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn openid_redirect(
    State(_db): State<DatabaseConnection>,
    ExtractSession {
        extractor: _form,
        session,
    }: ExtractSession<CsrfSafeForm<CreateProjectPayload>>,
) -> Result<impl IntoResponse, AppError> {
    let client = get_openid_client().await?;
    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    let session = session.lock().await;
    let pkce_verifier = session.openid_pkce_verifier();
    let nonce = session.openid_nonce();
    drop(session);

    // Now you can exchange it for an access token and ID token.
    let token_response = client
        .exchange_code(AuthorizationCode::new(
            "some authorization code".to_string(),
        ))
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await?;

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow!("Server did not return an ID token"))?;
    let claims = id_token.claims(&client.id_token_verifier(), &nonce)?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash =
            AccessTokenHash::from_token(token_response.access_token(), &id_token.signing_alg()?)?;
        if actual_access_token_hash != *expected_access_token_hash {
            Err(anyhow!("Invalid access token"))?;
        }
    }

    // The authenticated user's identity is now available. See the IdTokenClaims struct for a
    // complete listing of the available claims.
    println!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims
            .email()
            .map_or("<not provided>", |email| email.as_str())
    );
    Ok(Redirect::to("/list").into_response())
}
