use alloc::sync::Arc;

use axum::extract::multipart::MultipartError;
use axum::extract::rejection::FormRejection;
use axum::response::{Html, IntoResponse};
use handlebars::Handlebars;
use hyper::StatusCode;
use oauth2::basic::BasicErrorResponseType;
use oauth2::{RequestTokenError, StandardErrorResponse};
use once_cell::sync::Lazy;
use openidconnect::{ClaimsVerificationError, DiscoveryError, SigningError};
use serde::Serialize;

use crate::HANDLEBARS;

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
    #[error("IO error: {0}")]
    File(#[from] std::io::Error),
    #[error("bundling error: {0}")]
    Bundling(#[from] lightningcss::error::Error<String>),
    #[error("bundling error type 2: {0}")]
    Bundling2(#[from] lightningcss::error::Error<lightningcss::error::PrinterErrorKind>),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("webserver error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("template error: {0}")]
    Template(#[from] Box<handlebars::TemplateError>),
    #[error("unknown error: {0}")]
    Other(#[from] anyhow::Error),
    #[error("env var error: {0}")]
    EnvVar(#[from] std::env::VarError),
    #[error("rustls error: {0}")]
    Rustls(#[from] tokio_rustls::rustls::Error),
    #[error("wrong csrf token")]
    WrongCsrfToken,
    #[error("no accept remaining")]
    NoAcceptRemaining,
    #[error(
        "The request session is still held onto. Maybe you keep it alive inside a streaming \
         response?"
    )]
    SessionStillHeld,
    #[error(
        "Höchstwahrscheinlich ist deine Anmeldesession abgelaufen und du musst es erneut \
         versuchen. Wenn dies wieder auftritt, melde das Problem bitte an einen \
         Serveradministrator."
    )]
    OpenIdTokenNotFound,
}

#[derive(Serialize)]
pub struct ErrorTemplate {
    csrf_token: String,
    request_id: String,
    error: String,
}

pub struct AppErrorWithMetadata {
    pub csrf_token: String,
    pub request_id: String,
    pub app_error: AppError,
}

impl IntoResponse for AppErrorWithMetadata {
    fn into_response(self) -> axum::response::Response {
        match self.app_error {
            err @ (AppError::FormRejection(_)
            | AppError::Multipart(_)
            | AppError::Axum(_)
            | AppError::Database(_)
            | AppError::RequestToken(_)
            | AppError::ClaimsVerification(_)
            | AppError::Signing(_)
            | AppError::Discovery(_)
            | AppError::Oauth2Parse(_)
            | AppError::File(_)
            | AppError::Bundling(_)
            | AppError::Bundling2(_)
            | AppError::Json(_)
            | AppError::Hyper(_)
            | AppError::Template(_)
            | AppError::EnvVar(_)
            | AppError::Rustls(_)
            | AppError::OpenIdTokenNotFound
            | AppError::NoAcceptRemaining
            | AppError::SessionStillHeld
            | AppError::Other(_)) => {
                let result = HANDLEBARS
                    .render(
                        "error",
                        &ErrorTemplate {
                            csrf_token: self.csrf_token,
                            request_id: self.request_id,
                            error: err.to_string(),
                        },
                    )
                    .unwrap_or_else(|render_error| render_error.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(result).into_response(),
                )
                    .into_response()
            }
            err @ AppError::WrongCsrfToken => {
                (StatusCode::BAD_REQUEST, format!("{err}")).into_response()
            }
        }
    }
}
