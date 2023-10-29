use alloc::sync::Arc;

use axum::body::StreamBody;
use axum::extract::State;
use axum::response::IntoResponse;
use futures_async_stream::try_stream;
use futures_util::StreamExt;
use handlebars::Handlebars;
use html_escape::encode_safe;
use hyper::header;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
use serde_json::json;
use tokio::sync::Mutex;

use crate::csrf_protection::WithCsrfToken;
use crate::entities::project_history;
use crate::session::Session;
use crate::{EmptyBody, ExtractSession, TemplateProject, HANDLEBARS};

#[try_stream(ok = String, error = DbErr)]
async fn list_internal(db: DatabaseConnection, csrf_token: String) {
    let stream = project_history::Entity::find().stream(&db).await?;
    yield HANDLEBARS
        .render(
            "main_pre",
            &WithCsrfToken {
                csrf_token: &csrf_token,
                inner: json!({"page_title": "Projects"}),
            },
        )
        .unwrap_or_else(|render_error| render_error.to_string());
    #[for_await]
    for x in stream {
        let x = x?;
        let result = HANDLEBARS
            .render(
                "project",
                &TemplateProject {
                    title: x.title,
                    description: x.description,
                },
            )
            .unwrap_or_else(|render_error| render_error.to_string());
        yield result;
    }
    yield HANDLEBARS
        .render("main_post", &json!({}))
        .unwrap_or_else(|render_error| render_error.to_string());
}

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn list(
    State(db): State<DatabaseConnection>,
    ExtractSession {
        extractor: _form,
        session,
    }: ExtractSession<EmptyBody>,
) -> impl IntoResponse {
    let mut session_lock = session.lock().await;
    let session_id = session_lock.session().0;
    drop(session_lock);
    let stream = list_internal(db, session_id).map(|elem| match elem {
        Err(db_err) => Ok::<String, DbErr>(format!(
            "<h1>Error {}</h1>",
            encode_safe(&db_err.to_string())
        )),
        Ok::<String, DbErr>(ok) => Ok(ok),
    });
    (
        [(header::CONTENT_TYPE, "text/html")],
        StreamBody::new(stream),
    )
}
