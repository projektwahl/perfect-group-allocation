use anyhow::anyhow;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect};
use axum::{Form, TypedHeader};
use futures_util::TryFutureExt;
use handlebars::Handlebars;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse as OAuth2TokenResponse};
use openidconnect::{AccessTokenHash, TokenResponse as OpenIdTokenResponse};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppErrorWithMetadata};
use crate::openid::get_openid_client;
use crate::{CreateProjectPayload, CsrfSafeExtractor, CsrfSafeForm, ExtractSession, XRequestId};

// TODO FIXME check that form does an exact check and no unused inputs are accepted

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectError {
    state: String,
    error: String,
    error_description: String,
}

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectSuccess {
    state: String,
    session_state: String,
    code: String,
}

// TODO FIXME put common `state` directly into outer struct
#[derive(Deserialize)]
#[serde(untagged)]
pub enum OpenIdRedirect {
    Success(OpenIdRedirectSuccess),
    Error(OpenIdRedirectError),
}

// THIS IS DANGEROUS
impl CsrfSafeExtractor for Form<OpenIdRedirect> {}

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectErrorTemplate {
    csrf_token: String,
    error: String,
    error_description: String,
    state: String,
}

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn openid_redirect(
    State(handlebars): State<Handlebars<'static>>,
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    ExtractSession {
        extractor: form,
        session,
    }: ExtractSession<Form<OpenIdRedirect>>,
) -> Result<impl IntoResponse, AppErrorWithMetadata> {
    let mut session = session.lock().await;
    let expected_csrf_token = session.session_id();
    let result = async {
        // Once the user has been redirected to the redirect URL, you'll have access to the
        // authorization code. For security reasons, your code should verify that the `state`
        // parameter returned by the server matches `csrf_state`.
        let client = get_openid_client().await?;

        match form.0 {
            OpenIdRedirect::Error(err) => {
                let csrf_token = session.session_id();
                let openid_csrf_token = session.openid_csrf_token();
                drop(session);

                assert_eq!(&err.state, openid_csrf_token.secret());

                let result = handlebars
                    .render(
                        "openid_redirect",
                        &OpenIdRedirectErrorTemplate {
                            csrf_token,
                            error: err.error,
                            error_description: err.error_description,
                            state: err.state,
                        },
                    )
                    .unwrap_or_else(|e| e.to_string());
                Ok(Html(result).into_response())
            }
            OpenIdRedirect::Success(ok) => {
                let pkce_verifier = session.openid_pkce_verifier();
                let nonce = session.openid_nonce();
                let openid_csrf_token = session.openid_csrf_token();
                drop(session);

                assert_eq!(&ok.state, openid_csrf_token.secret());

                // Now you can exchange it for an access token and ID token.
                let token_response = client
                    .exchange_code(AuthorizationCode::new(ok.code))
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
                    let actual_access_token_hash = AccessTokenHash::from_token(
                        token_response.access_token(),
                        &id_token.signing_alg()?,
                    )?;
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
        }
    };
    result
        .or_else(|app_error| async {
            // TODO FIXME store request id type-safe in body/session
            Err(AppErrorWithMetadata {
                csrf_token: expected_csrf_token.clone(),
                request_id,
                app_error,
            })
        })
        .await
}
