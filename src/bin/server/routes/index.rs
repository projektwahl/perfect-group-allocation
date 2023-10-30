use alloc::sync::Arc;
use std::sync::PoisonError;

use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::TypedHeader;
use handlebars::Handlebars;
use once_cell::sync::Lazy;

use crate::error::AppErrorWithMetadata;
use crate::templating::render;
use crate::{CreateProject, EmptyBody, ExtractSession, XRequestId, HANDLEBARS};

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn index(
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    ExtractSession { session, .. }: ExtractSession<EmptyBody>,
) -> impl IntoResponse {
    let result = async {
        let result = render(
            &session,
            "create-project",
            &CreateProject {
                title: None,
                title_error: None,
                description: None,
                description_error: None,
            },
        );
        Ok(Html(result))
    };
    match result.await {
        Ok(ok) => Ok(ok),
        Err(app_error) => {
            // TODO FIXME store request id type-safe in body/session
            Err(AppErrorWithMetadata {
                session,
                request_id,
                app_error,
            })
        }
    }
}
