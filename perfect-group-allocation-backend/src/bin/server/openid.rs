use std::sync::OnceLock;

use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::reqwest::async_http_client;
use oauth2::{
    ClientId, ClientSecret, EmptyExtraTokenFields, RedirectUrl, RevocationErrorResponseType,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse,
};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreClient, CoreGenderClaim, CoreJsonWebKey,
    CoreJsonWebKeyType, CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm, CoreProviderMetadata,
};
use openidconnect::{EmptyAdditionalClaims, IdTokenFields, IssuerUrl};

use crate::error::AppError;

pub static OPENID_CLIENT: OnceLock<
    Result<
        openidconnect::Client<
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
        >,
        AppError,
    >,
> = OnceLock::new();

pub async fn initialize_favicon_ico() {
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
