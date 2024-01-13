pub mod error;

use std::sync::OnceLock;

use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::reqwest::async_http_client;
pub use oauth2::RefreshToken;
use oauth2::{
    ClientId, ClientSecret, EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, RevocationErrorResponseType, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreGenderClaim,
    CoreJsonWebKey, CoreJsonWebKeyType, CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm, CoreProviderMetadata,
};
pub use openidconnect::EndUserEmail;
use openidconnect::{EmptyAdditionalClaims, IdTokenFields, IssuerUrl, Nonce};
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

#[derive(Serialize, Deserialize)]
pub struct OpenIdSession {
    verifier: PkceCodeVerifier,
    nonce: Nonce,
    csrf_token: oauth2::CsrfToken,
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

pub async fn begin_authentication() {
    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = get_openid_client()
        .await
        .unwrap()
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            openidconnect::CsrfToken::new_random,
            Nonce::new_random,
        )
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();
}
