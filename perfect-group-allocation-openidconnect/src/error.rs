use oauth2::basic::BasicErrorResponseType;
use oauth2::{RequestTokenError, StandardErrorResponse};
use openidconnect::{ClaimsVerificationError, DiscoveryError, SigningError};

#[derive(thiserror::Error, Debug)]
pub enum OpenIdConnectError {
    #[error("request token error: {0}")]
    RequestToken(
        #[from] RequestTokenError<HttpError, StandardErrorResponse<BasicErrorResponseType>>,
    ),
    #[error("claims verification error: {0}")]
    ClaimsVerification(#[from] ClaimsVerificationError),
    #[error("openid signing error: {0}")]
    Signing(#[from] SigningError),
    #[error("oauth error: {0}")]
    Oauth2Parse(#[from] oauth2::url::ParseError),
    #[error("discovery error: {}", match .0 { openidconnect::DiscoveryError::<HttpError>::Request(req) => format!("{req}"), other => format!("{other}") })]
    Discovery(#[from] DiscoveryError<HttpError>),
    #[error("wrong csrf token")]
    WrongCsrfToken,
    #[error("server did not return id token")]
    NoIdTokenReturned,
    #[error("invalid access token")]
    InvalidAccessToken,
    #[error("missing email address")]
    MissingEmailAddress,
}

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("hyper {0}")]
    Hyper(#[from] hyper::Error),
    #[error("hyper http {0}")]
    HyperHttp(#[from] hyper::http::Error),
    #[error("io {0}")]
    Io(#[from] std::io::Error),
}
