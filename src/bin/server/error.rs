use axum::extract::multipart::MultipartError;
use axum::extract::rejection::FormRejection;
use axum::response::IntoResponse;
use hyper::StatusCode;
use oauth2::basic::BasicErrorResponseType;
use oauth2::{RequestTokenError, StandardErrorResponse};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("form submission error {0}")]
    FormRejection(#[from] FormRejection),
    #[error("form upload error {0}")]
    Multipart(#[from] MultipartError),
    #[error("webserver error {0}")]
    Axum(#[from] axum::Error),
    #[error("database error {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("request token error")]
    RequestToken(
        #[from]
        RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    ),
    #[error("unknown error {0}")]
    Other(#[from] anyhow::Error),
    #[error("wrong csrf token")]
    WrongCsrfToken,
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            err @ (AppError::FormRejection(_)
            | AppError::Multipart(_)
            | AppError::Axum(_)
            | AppError::Database(_)
            | AppError::RequestToken(_)
            | AppError::Other(_)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", err),
            )
                .into_response(),
            err @ AppError::WrongCsrfToken => {
                (StatusCode::BAD_REQUEST, format!("{}", err)).into_response()
            }
        }
    }
}
