use alloc::sync::Arc;

use axum::extract::State;
use axum::response::{Html, IntoResponse};
use handlebars::Handlebars;
use once_cell::sync::Lazy;

use crate::{CreateProject, EmptyBody, ExtractSession, HANDLEBARS};

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn index(ExtractSession { session, .. }: ExtractSession<EmptyBody>) -> impl IntoResponse {
    let result = HANDLEBARS
        .render(
            "create-project",
            &CreateProject {
                csrf_token: session.lock().await.session().0,
                title: None,
                title_error: None,
                description: None,
                description_error: None,
            },
        )
        .unwrap_or_else(|render_error| render_error.to_string());
    Html(result)
}
