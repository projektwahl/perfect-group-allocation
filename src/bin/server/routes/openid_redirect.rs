use anyhow::anyhow;
use axum::response::{Html, IntoResponse, Redirect};
use axum::TypedHeader;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse as OAuth2TokenResponse};
use openidconnect::{AccessTokenHash, TokenResponse as OpenIdTokenResponse};
use serde::{Deserialize, Serialize};

use crate::error::{to_error_result, AppError};
use crate::openid::get_openid_client;
use crate::session::{Session, SessionCookie};
use crate::templating::render;
use crate::XRequestId;

// TODO FIXME check that form does an exact check and no unused inputs are accepted

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectError {
    error: String,
    error_description: String,
}

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectSuccess {
    session_state: String,
    code: String,
}

#[derive(Deserialize)]
pub struct OpenIdRedirect {
    state: String,
    #[serde(flatten)]
    inner: OpenIdRedirectInner,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum OpenIdRedirectInner {
    Success(OpenIdRedirectSuccess),
    Error(OpenIdRedirectError),
}

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectErrorTemplate {
    csrf_token: String,
    error: String,
    error_description: String,
}

#[expect(
    clippy::disallowed_types,
    reason = "csrf protection done here explicitly"
)]
#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn openid_redirect(
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    mut session: Session, // what if this here could be a reference?
    form: axum::Form<OpenIdRedirect>,
) -> Result<(Session, impl IntoResponse), (Session, impl IntoResponse)> {
    let session_ref = &mut session;
    let result = async {
        // what if privatecookiejar (and session?) would be non-owning (I don't want to clone them)
        // TODO FIXME errors also need to return the session?

        let expected_csrf_token = session_ref.session().0;
        let (pkce_verifier, nonce, openid_csrf_token) =
            session_ref.get_and_remove_openidconnect()?;

        if &form.0.state != openid_csrf_token.secret() {
            return Err(AppError::WrongCsrfToken);
        };

        // Once the user has been redirected to the redirect URL, you'll have access to the
        // authorization code. For security reasons, your code should verify that the `state`
        // parameter returned by the server matches `csrf_state`.
        let client = get_openid_client().await?;

        match form.0.inner {
            OpenIdRedirectInner::Error(err) => {
                let result = render(
                    session_ref,
                    "openid_redirect",
                    OpenIdRedirectErrorTemplate {
                        csrf_token: expected_csrf_token.clone(),
                        error: err.error,
                        error_description: err.error_description,
                    },
                )
                .await;
                Ok(Html(result).into_response())
            }
            OpenIdRedirectInner::Success(ok) => {
                // TODO FIXME isn't it possible to directly get the id token?
                // maybe the other way the client also gets the data / the browser history (but I would think its encrypted)

                // this way we may also be able to use the refresh token? (would be nice for mobile performance)

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
                        return Err(anyhow!("Invalid access token").into());
                    }
                }

                let Some(email) = claims.email() else {
                    return Err(anyhow!("No email address received by SSO").into());
                };

                // TODO FIXME our application should work without refresh token
                let Some(refresh_token) = token_response.refresh_token() else {
                    return Err(anyhow!("No refresh token received by SSO").into());
                };

                session_ref.set_session(Some(SessionCookie {
                    email: email.to_owned(),
                    expiration: claims.expiration(),
                    refresh_token: refresh_token.to_owned(),
                }));

                Ok(Redirect::to("/list").into_response())
            }
        }
    };
    match result.await {
        Ok(ok) => Ok((session, ok)),
        Err(app_error) => Err(to_error_result(session, request_id, app_error).await),
    }
}
