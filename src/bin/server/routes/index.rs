use alloc::sync::Arc;
use std::sync::PoisonError;

use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::TypedHeader;
use handlebars::Handlebars;
use once_cell::sync::Lazy;

use crate::error::AppErrorWithMetadata;
use crate::{CreateProject, EmptyBody, ExtractSession, XRequestId, HANDLEBARS};

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn index(
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    ExtractSession { session, .. }: ExtractSession<EmptyBody>,
) -> impl IntoResponse {
    let result = async {
        let mut session_lock = session.lock().map_err(|p| PoisonError::new(()))?;
        let result = HANDLEBARS
            .render(
                "create-project",
                &CreateProject {
                    csrf_token: session_lock.session().0,
                    title: None,
                    title_error: None,
                    description: None,
                    description_error: None,
                },
            )
            .unwrap_or_else(|render_error| render_error.to_string());
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
