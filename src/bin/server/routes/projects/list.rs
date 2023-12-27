use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::StreamExt;
use hyper::header;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
use serde_json::json;

use crate::entities::project_history;
use crate::error::AppError;
use crate::session::Session;
use crate::templating::render;
use crate::{TemplateProject, XRequestId};

async gen fn list_internal(db: DatabaseConnection, session: Session) -> Result<String, AppError> {
    let stream = project_history::Entity::find().stream(&db).await?;
    yield render(&session, "main_pre", json!({"page_title": "Projects"})).await;
    #[for_await]
    for x in stream {
        let x = x?;
        let result = render(
            &session,
            "project",
            TemplateProject {
                title: x.title,
                description: x.description,
            },
        )
        .await;
        yield result;
    }
    yield render(&session, "main_post", json!({})).await;
}

#[axum::debug_handler(state=crate::MyState)]
pub async fn list(
    State(db): State<DatabaseConnection>,
    TypedHeader(XRequestId(_request_id)): TypedHeader<XRequestId>,
    session: Session,
) -> (Session, impl IntoResponse) {
    let stream = list_internal(db, session.clone()).map(|elem| match elem {
        Err(app_error) => Ok::<String, AppError>(format!(
            // TODO FIXME use template here
            "<h1>Error {}</h1>",
            encode_safe(&app_error.to_string())
        )),
        Ok::<String, AppError>(ok) => Ok(ok),
    });
    (
        session,
        (
            [(header::CONTENT_TYPE, "text/html")],
            StreamBody::new(stream),
        ),
    )
}
