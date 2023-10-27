use axum::extract::State;
use axum::response::{Html, IntoResponse};
use handlebars::Handlebars;

use crate::{CreateProject, EmptyBody, ExtractSession, MyBody, MyState};

#[axum::debug_handler(body=MyBody, state=MyState)]
pub async fn index(
    handlebars: State<Handlebars<'static>>,
    ExtractSession {
        extractor: _,
        session,
    }: ExtractSession<EmptyBody>,
) -> impl IntoResponse {
    let result = handlebars
        .render(
            "create-project",
            &CreateProject {
                csrf_token: session.lock().await.session_id(),
                title: None,
                title_error: None,
                description: None,
                description_error: None,
            },
        )
        .unwrap_or_else(|e| e.to_string());
    Html(result)
}
