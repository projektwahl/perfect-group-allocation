use alloc::sync::Arc;
use std::sync::{Mutex, PoisonError};

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
use crate::templating::render;
use crate::{EmptyBody, ExtractSession, TemplateProject, XRequestId, HANDLEBARS};

#[try_stream(ok = String, error = DbErr)]
async fn list_internal(db: DatabaseConnection, session: Session) {
    let stream = project_history::Entity::find().stream(&db).await?;
    yield render(&session, "main_pre", json!({"page_title": "Projects"}));
    #[for_await]
    for x in stream {
        let x = x?;
        let result = render(
            &session,
            "project",
            &TemplateProject {
                title: x.title,
                description: x.description,
            },
        );
        yield result;
    }
    yield render(&session, "main_post", &json!({}));
}

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn list(
    State(db): State<DatabaseConnection>,
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    ExtractSession {
        extractor: _form,
        session,
    }: ExtractSession<EmptyBody>,
) -> (Session, impl IntoResponse) {
    let stream = list_internal(db, session.clone()).map(|elem| match elem {
        Err(db_err) => Ok::<String, DbErr>(format!(
            // TODO FIXME use template here
            "<h1>Error {}</h1>",
            encode_safe(&db_err.to_string())
        )),
        Ok::<String, DbErr>(ok) => Ok(ok),
    });
    (
        session,
        (
            [(header::CONTENT_TYPE, "text/html")],
            StreamBody::new(stream),
        ),
    )
}
