use alloc::sync::Arc;
use std::sync::PoisonError;

use axum::body::StreamBody;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::TypedHeader;
use futures_async_stream::try_stream;
use futures_util::StreamExt;
use handlebars::Handlebars;
use html_escape::encode_safe;
use hyper::header;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
use serde_json::json;

use crate::csrf_protection::WithCsrfToken;
use crate::entities::project_history;
use crate::error::AppErrorWithMetadata;
use crate::session::Session;
use crate::{EmptyBody, ExtractSession, TemplateProject, XRequestId, HANDLEBARS};

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
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    ExtractSession {
        extractor: _form,
        session,
    }: ExtractSession<EmptyBody>,
) -> Result<impl IntoResponse, AppErrorWithMetadata> {
    let result = async {
        let mut session_lock = session.lock().map_err(|p| PoisonError::new(()))?;
        let session_id = session_lock.session().0;
        drop(session_lock);
        let stream = list_internal(db, session_id).map(|elem| match elem {
            Err(db_err) => Ok::<String, DbErr>(format!(
                "<h1>Error {}</h1>",
                encode_safe(&db_err.to_string())
            )),
            Ok::<String, DbErr>(ok) => Ok(ok),
        });
        Ok((
            [(header::CONTENT_TYPE, "text/html")],
            StreamBody::new(stream),
        ))
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
