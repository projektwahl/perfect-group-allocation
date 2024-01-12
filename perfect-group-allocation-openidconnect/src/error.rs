use oauth2::basic::BasicErrorResponseType;
use oauth2::{RequestTokenError, StandardErrorResponse};
use openidconnect::{ClaimsVerificationError, DiscoveryError, SigningError};

#[derive(thiserror::Error, Debug)]
pub enum OpenIdConnectError {
    #[error("request token error: {0}")]
    RequestToken(
        #[from]
        RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    ),
    #[error("claims verification error: {0}")]
    ClaimsVerification(#[from] ClaimsVerificationError),
    #[error("openid signing error: {0}")]
    Signing(#[from] SigningError),
    #[error("oauth error: {0}")]
    Oauth2Parse(#[from] oauth2::url::ParseError),
    #[error("discovery error: {0}")]
    Discovery(#[from] DiscoveryError<oauth2::reqwest::Error<reqwest::Error>>),
}
