use alloc::borrow::Cow;

use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use bytes::Bytes;
use futures_util::StreamExt;
use hyper::header;
use sea_orm::{DatabaseConnection, EntityTrait};
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{template_stream, yieldoki, yieldokv};

use crate::entities::project_history;
use crate::error::AppError;
use crate::session::Session;

#[template_stream("templates")]
async gen fn list_internal(
    db: DatabaseConnection,
    session: Session,
) -> Result<alloc::borrow::Cow<'static, str>, AppError> {
    let template = yieldoki!(list_projects());
    let template = yieldoki!(template.next());
    let template = yieldoki!(template.next());
    let template = yieldokv!(template.page_title("Projects"));
    let template = yieldoki!(template.next());
    let template = yieldoki!(template.next());
    let template = yieldoki!(template.next_email_false());
    let template = yieldokv!(template.csrf_token(session.session().0));
    let template = yieldoki!(template.next());
    let template = yieldoki!(template.next());
    let mut template = yieldoki!(template.next());
    let mut stream = project_history::Entity::find().stream(&db).await.unwrap();
    while let Some(x) = stream.next().await {
        let inner_template = yieldoki!(template.next_enter_loop());
        let x = x.unwrap();
        let inner_template = yieldokv!(inner_template.title(x.title));
        let inner_template = yieldoki!(inner_template.next());
        let inner_template = yieldokv!(inner_template.description(x.description));
        template = yieldoki!(inner_template.next());
    }
    let template = yieldoki!(template.next_end_loop());
    yieldoki!(template.next());
}

#[axum::debug_handler(state=crate::MyState)]
pub async fn list(
    State(db): State<DatabaseConnection>,
    session: Session,
) -> (Session, impl IntoResponse) {
    let stream = AsyncIteratorStream(list_internal(db, session.clone())).map(|elem| match elem {
        Err(app_error) => Ok::<Bytes, AppError>(Bytes::from(format!(
            // TODO FIXME use template here
            "<h1>Error {}</h1>",
            &app_error.to_string()
        ))),
        Ok(Cow::Owned(ok)) => Ok::<Bytes, AppError>(Bytes::from(ok)),
        Ok(Cow::Borrowed(ok)) => Ok::<Bytes, AppError>(Bytes::from(ok)),
    });
    (
        session,
        (
            [(header::CONTENT_TYPE, "text/html")],
            axum::body::Body::from_stream(stream),
        ),
    )
}
