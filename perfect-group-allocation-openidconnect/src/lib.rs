pub mod error;

use std::str::FromStr;
use std::sync::Arc;

use error::HttpError;
use http_body_util::BodyExt;
use hyper_util::rt::TokioIo;
use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
pub use oauth2::RefreshToken;
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, EmptyExtraTokenFields, HttpRequest, HttpResponse,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RevocationErrorResponseType,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse as _,
};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreGenderClaim,
    CoreIdToken, CoreJsonWebKey, CoreJsonWebKeyType, CoreJsonWebKeyUse,
    CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreProviderMetadata,
};
pub use openidconnect::EndUserEmail;
use openidconnect::{
    AccessTokenHash, EmptyAdditionalClaims, IdTokenClaims, IdTokenFields, IssuerUrl, Nonce,
    TokenResponse,
};
use perfect_group_allocation_config::Config;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::OnceCell;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;

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

pub async fn my_http_client(request: HttpRequest) -> Result<HttpResponse, HttpError> {
    println!("{:?}", request);
    let url = request.url;
    let host = url.host().expect("uri has no host");
    let port = url.port_or_known_default().unwrap();
    let addr = format!("{host}:{port}");

    // TODO FIXME self signed certs
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let dnsname = ServerName::try_from(host.to_string()).unwrap();

    let stream = TcpStream::connect(addr).await?;
    let stream = connector.connect(dnsname, stream).await?;

    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move { if let Err(_err) = conn.await {} });

    let authority = url.authority();

    let request = hyper::Request::builder()
        .uri(url.to_string())
        .header(hyper::header::HOST, authority)
        .body(String::new())?;

    let response = sender.send_request(request).await?;

    println!("{:?}", response);

    Ok(HttpResponse {
        // this is http 0.2
        status_code: oauth2::http::StatusCode::from_u16(response.status().as_u16()).unwrap(),
        headers: response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    oauth2::http::HeaderName::from_str(name.as_str()).unwrap(),
                    oauth2::http::HeaderValue::from_bytes(value.as_bytes()).unwrap(),
                )
            })
            .collect(),
        body: response.collect().await?.to_bytes().to_vec(),
    })
}

#[allow(unused)]
pub async fn get_openid_client(
    config: &Config,
) -> Result<&'static OpenIdConnectClientType, OpenIdConnectError> {
    OPENID_CLIENT
        .get_or_try_init(|| async {
            let provider_metadata = CoreProviderMetadata::discover_async(
                IssuerUrl::new(config.openidconnect.issuer_url.clone())?,
                my_http_client,
            )
            .await?;

            // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
            // and token URL.
            let client = CoreClient::from_provider_metadata(
                provider_metadata,
                ClientId::new(config.openidconnect.client_id.clone()),
                Some(ClientSecret::new(
                    config.openidconnect.client_secret.clone(),
                )),
            )
            // Set the URL the user will be redirected to after the authorization process.
            .set_redirect_uri(RedirectUrl::new(format!(
                "{}/openidconnect-redirect",
                config.url
            ))?);
            Ok(client)
        })
        .await
}

pub async fn begin_authentication(
    config: &Config,
) -> Result<(String, OpenIdSession), OpenIdConnectError> {
    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = get_openid_client(config)
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
    config: &Config,
    session: OpenIdSession,
    input: OpenIdRedirect<OpenIdRedirectSuccess>,
) -> Result<String, OpenIdConnectError> {
    if &input.state != session.csrf_token.secret() {
        return Err(OpenIdConnectError::WrongCsrfToken);
    };

    let client = get_openid_client(config).await?;

    // TODO FIXME isn't it possible to directly get the id token?
    // maybe the other way the client also gets the data / the browser history (but I would think its encrypted)

    // this way we may also be able to use the refresh token? (would be nice for mobile performance)

    // Now you can exchange it for an access token and ID token.
    let token_response = client
        .exchange_code(AuthorizationCode::new(input.inner.code))
        // Set the PKCE code verifier.
        .set_pkce_verifier(session.verifier)
        .request_async(my_http_client)
        .await?;

    // the token_response may be signed and then we could store it in the cookie

    // TODO FIXME store it in cookie?

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response
        .id_token()
        .ok_or_else(|| OpenIdConnectError::NoIdTokenReturned)?;
    let claims = id_token.claims(&client.id_token_verifier(), &session.nonce)?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash =
            AccessTokenHash::from_token(token_response.access_token(), &id_token.signing_alg()?)?;
        if actual_access_token_hash != *expected_access_token_hash {
            return Err(OpenIdConnectError::InvalidAccessToken);
        }
    }

    println!("{claims:?}");

    let Some(_email) = claims.email() else {
        return Err(OpenIdConnectError::MissingEmailAddress);
    };

    // TODO FIXME our application should work without refresh token but use it for efficiency?
    // token_response.refresh_token()

    Ok(serde_json::to_string(id_token).unwrap())
}

pub async fn id_token_claims(
    config: &Config,
    id_token: String,
) -> Result<IdTokenClaims<EmptyAdditionalClaims, CoreGenderClaim>, OpenIdConnectError> {
    let client = get_openid_client(config).await?;

    let id_token: CoreIdToken = serde_json::from_str(&id_token).unwrap();
    let claims = id_token.claims(&client.id_token_verifier(), |_v: Option<&Nonce>| Ok(()))?;
    Ok(claims.clone())
}
