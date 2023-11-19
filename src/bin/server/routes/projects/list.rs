use alloc::borrow::Cow;

use axum::body::StreamBody;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::TypedHeader;
use futures_async_stream::try_stream;
use futures_util::StreamExt;
use html_escape::encode_safe;
use hyper::header;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
use serde_json::json;
use zero_cost_templating::template_stream;

use crate::entities::project_history;
use crate::error::AppError;
use crate::session::Session;
use crate::templating::render;
use crate::{TemplateProject, XRequestId};

#[template_stream("templates/list.html.hbs")]
pub async fn test2(db: DatabaseConnection, session: Session) {
    let template = initial0!();
    let template = template0!(template);
    let page_title = "Projects";
    let template = page_title1!(template, page_title);
    let csrf_token = session.session().0;
    let mut template = csrf_token2!(template, csrf_token);
    let stream = project_history::Entity::find().stream(&db).await.unwrap();
    #[for_await]
    for x in stream {
        let x = x.unwrap();
        let inner_template = template3!(template);
        let inner_template = title4!(inner_template, x.title);
        template = description5!(inner_template, x.description);
    }
    template6!(template);
}

#[try_stream(ok = String, error = AppError)]
async fn list_internal(db: DatabaseConnection, session: Session) {
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

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn list(
    State(db): State<DatabaseConnection>,
    TypedHeader(XRequestId(_request_id)): TypedHeader<XRequestId>,
    session: Session,
) -> (Session, impl IntoResponse) {
    let stream = test2(db, session.clone()).map(|elem| Ok::<String, AppError>(elem.to_string())); /*.map(|elem| match elem {
    Err(app_error) => Ok::<String, AppError>(format!(
    // TODO FIXME use template here
    "<h1>Error {}</h1>",
    encode_safe(&app_error.to_string())
    )),
    Ok::<String, AppError>(ok) => Ok(ok),
    });*/
    (
        session,
        (
            [(header::CONTENT_TYPE, "text/html")],
            StreamBody::new(stream),
        ),
    )
}
