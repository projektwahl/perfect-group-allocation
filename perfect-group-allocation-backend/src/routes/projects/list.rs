use alloc::borrow::Cow;

use bytes::Bytes;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use headers::ContentType;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::StreamBody;
use hyper::header;
use perfect_group_allocation_database::models::ProjectHistoryEntry;
use perfect_group_allocation_database::schema::project_history;
use perfect_group_allocation_database::{DatabaseConnection, Pool};
use tracing::error;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{template_stream, yieldoki, yieldokv};

use crate::error::AppError;
use crate::routes::list_projects;
use crate::session::Session;
use crate::ResponseTypedHeaderExt as _;

async gen fn list_internal(
    pool: Pool,
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
    let mut connection = match pool.get().await {
        Ok(connection) => connection,
        Err(erroro) => todo!(),
    };
    let mut stream = match project_history::table
        .group_by((
            project_history::id,
            project_history::title,
            project_history::info,
        ))
        .select((
            project_history::id,
            diesel::dsl::max(project_history::history_id).assume_not_null(),
            project_history::title,
            project_history::info,
        ))
        .load_stream::<ProjectHistoryEntry>(&mut connection)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            error!("{:?}", error);
            let template = yieldoki!(template.next_end_loop());
            yieldoki!(template.next());
            return; // TODO FIXME
        }
    };
    while let Some(x) = stream.next().await {
        let inner_template = yieldoki!(template.next_enter_loop());
        let x = x.unwrap();
        let inner_template = yieldokv!(inner_template.title(x.title));
        let inner_template = yieldoki!(inner_template.next());
        let inner_template = yieldokv!(inner_template.description(x.info));
        template = yieldoki!(inner_template.next());
    }
    let template = yieldoki!(template.next_end_loop());
    yieldoki!(template.next());
}

pub async fn list(
    db: DatabaseConnection,
    session: Session,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = AppError>>, AppError> {
    let stream = AsyncIteratorStream(list_internal(db, session.clone())).map(|elem| match elem {
        Err(app_error) => Ok::<Bytes, AppError>(Bytes::from(format!(
            // TODO FIXME use template here
            "<h1>Error {}</h1>",
            &app_error.to_string()
        ))),
        Ok(Cow::Owned(ok)) => Ok::<Bytes, AppError>(Bytes::from(ok)),
        Ok(Cow::Borrowed(ok)) => Ok::<Bytes, AppError>(Bytes::from(ok)),
    });
    Ok(Response::builder()
        .status(StatusCode::OK)
        .typed_header(ContentType::html())
        .body(StreamBody::new(stream))
        .unwrap())
}
