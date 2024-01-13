pub mod error;

use std::sync::OnceLock;

use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::reqwest::async_http_client;
pub use oauth2::RefreshToken;
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, EmptyExtraTokenFields, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RevocationErrorResponseType, StandardErrorResponse,
    StandardRevocableToken, StandardTokenIntrospectionResponse, StandardTokenResponse,
    TokenResponse as _,
};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreGenderClaim,
    CoreJsonWebKey, CoreJsonWebKeyType, CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm, CoreProviderMetadata,
};
pub use openidconnect::EndUserEmail;
use openidconnect::{
    AccessTokenHash, EmptyAdditionalClaims, IdTokenFields, IssuerUrl, Nonce, TokenResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{OnceCell, RwLock};

use crate::error::OpenIdConnectError;

type OpenIdConnectClientType = openidconnect::Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<
        IdTokenFields<
            EmptyAdditionalClaims,
            EmptyExtraTokenFields,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
            CoreJsonWebKeyType,
        >,
        BasicTokenType,
    >,
    BasicTokenType,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
>;

#[derive(Deserialize)]
pub struct OpenIdRedirect<T> {
    pub state: String,
    #[serde(flatten)]
    pub inner: T,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum OpenIdRedirectInner {
    Success(OpenIdRedirectSuccess),
    Error(OpenIdRedirectError),
}

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectError {
    pub error: String,
    pub error_description: String,
}

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectSuccess {
    pub session_state: String,
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct OpenIdSession {
    pub verifier: PkceCodeVerifier,
    pub nonce: Nonce,
    pub csrf_token: oauth2::CsrfToken,
}

static OPENID_CLIENT: OnceCell<OpenIdConnectClientType> = OnceCell::const_new();

#[allow(unused)]
// TODO FIXME initialize on request, to make it not fail on intermittend failures. cache response (forever?)
pub async fn get_openid_client() -> Result<&'static OpenIdConnectClientType, OpenIdConnectError> {
    OPENID_CLIENT
        .get_or_try_init(|| async {
            let provider_metadata = CoreProviderMetadata::discover_async(
                IssuerUrl::new("http://localhost:8080/realms/pga".to_owned())?,
                async_http_client,
            )
            .await?;

            // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
            // and token URL.
            let client = CoreClient::from_provider_metadata(
                provider_metadata,
                ClientId::new("pga".to_owned()),
                Some(ClientSecret::new(
                    "cGRSAwBaSfTENHt7npPrsAfcqqWM1uqU".to_owned(),
                )),
            )
            // Set the URL the user will be redirected to after the authorization process.
            .set_redirect_uri(RedirectUrl::new(
                "http://localhost:3000/openidconnect-redirect".to_owned(),
            )?);
            Ok(client)
        })
        .await
}

pub async fn begin_authentication() -> Result<(String, OpenIdSession), OpenIdConnectError> {
    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = get_openid_client()
        .await?
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            openidconnect::CsrfToken::new_random,
            Nonce::new_random,
        )
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    Ok((
        auth_url.to_string(),
        OpenIdSession {
            verifier: pkce_verifier,
            nonce,
            csrf_token,
        },
    ))
}

pub async fn finish_authentication(
    session: OpenIdSession,
    input: OpenIdRedirect<OpenIdRedirectSuccess>,
) -> Result<String, OpenIdConnectError> {
    if &input.state != session.csrf_token.secret() {
        return Err(OpenIdConnectError::WrongCsrfToken);
    };

    let client = get_openid_client().await?;

    // TODO FIXME isn't it possible to directly get the id token?
    // maybe the other way the client also gets the data / the browser history (but I would think its encrypted)

    // this way we may also be able to use the refresh token? (would be nice for mobile performance)

    // Now you can exchange it for an access token and ID token.
    let token_response = client
        .exchange_code(AuthorizationCode::new(input.inner.code))
        // Set the PKCE code verifier.
        .set_pkce_verifier(session.verifier)
        .request_async(async_http_client)
        .await?;

    // the token_response may be signed and then we could store it in the cookie

    // TODO FIXME store it in cookie?

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

    Ok("hi".to_owned())
}
