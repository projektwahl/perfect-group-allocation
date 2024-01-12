pub mod error;

use std::sync::OnceLock;

use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::reqwest::async_http_client;
pub use oauth2::RefreshToken;
use oauth2::{
    ClientId, ClientSecret, EmptyExtraTokenFields, PkceCodeVerifier, RedirectUrl,
    RevocationErrorResponseType, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreClient, CoreGenderClaim, CoreJsonWebKey,
    CoreJsonWebKeyType, CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm, CoreProviderMetadata,
};
pub use openidconnect::EndUserEmail;
use openidconnect::{EmptyAdditionalClaims, IdTokenFields, IssuerUrl, Nonce};
use serde::{Deserialize, Serialize};

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

pub static OPENID_CLIENT: OnceLock<Result<OpenIdConnectClientType, OpenIdConnectError>> =
    OnceLock::new();

#[allow(unused)]
pub async fn initialize_openid_client() {
    let client = async {
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
            "https://localhost:8443/openidconnect-redirect".to_owned(),
        )?);
        Ok(client)
    };

    OPENID_CLIENT.set(client.await).unwrap();
}
