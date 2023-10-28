use axum::extract::multipart::MultipartError;
use axum::extract::rejection::FormRejection;
use axum::response::{Html, IntoResponse};
use hyper::StatusCode;
use oauth2::basic::BasicErrorResponseType;
use oauth2::{RequestTokenError, StandardErrorResponse};
use openidconnect::{ClaimsVerificationError, DiscoveryError, SigningError};
use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("form submission error: {0}")]
    FormRejection(#[from] FormRejection),
    #[error("form upload error: {0}")]
    Multipart(#[from] MultipartError),
    #[error("webserver error: {0}")]
    Axum(#[from] axum::Error),
    #[error("database error: {0}")]
    Database(#[from] sea_orm::DbErr),
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
    #[error("unknown error: {0}")]
    Other(#[from] anyhow::Error),
    #[error("wrong csrf token")]
    WrongCsrfToken,
}

#[derive(Serialize)]
pub struct ErrorTemplate {
    csrf_token: String,
    request_id: String,
    error: String,
}

/*
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            err @ (Self::FormRejection(_)
            | Self::Multipart(_)
            | Self::Axum(_)
            | Self::Database(_)
            | Self::RequestToken(_)
            | Self::ClaimsVerification(_)
            | Self::Signing(_)
            | Self::Discovery(_)
            | Self::Oauth2Parse(_)
            | Self::Other(_)) => {
                let result = HANDLEBARS
                    .render(
                        "error",
                        &ErrorTemplate {
                            csrf_token: "jo".to_string(),
                            request_id: "hi".to_string(),
                            error: "test".to_string(),
                        },
                    )
                    .unwrap_or_else(|e| e.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(result).into_response(),
                )
                    .into_response()
            }
            err @ Self::WrongCsrfToken => {
                (StatusCode::BAD_REQUEST, format!("{err}")).into_response()
            }
        }
    }
}
*/
