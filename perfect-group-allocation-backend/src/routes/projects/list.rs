
use std::convert::Infallible;

use bytes::Bytes;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use headers::ContentType;
use http::{Response, StatusCode};
use http_body::{Body, Frame};
use http_body_util::StreamBody;

use perfect_group_allocation_css::index_css;
use perfect_group_allocation_database::models::ProjectHistoryEntry;
use perfect_group_allocation_database::schema::project_history;
use perfect_group_allocation_database::{Pool};
use tracing::error;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{Unsafe};

use crate::error::AppError;
use crate::routes::list_projects;
use crate::session::Session;
use crate::{yieldfi, yieldfv, ResponseTypedHeaderExt as _};

async gen fn list_internal(pool: Pool, session: Session) -> Result<Frame<Bytes>, Infallible> {
    let template = yieldfi!(list_projects());
    let template = yieldfi!(template.next());
    let template = yieldfi!(template.next());
    let template = yieldfv!(template.page_title("Projects"));
    let template = yieldfi!(template.next());
    let template = yieldfv!(
        template.indexcss_version_unsafe(Unsafe::unsafe_input(index_css!().1.to_string()))
    );
    let template = yieldfi!(template.next());
    let template = yieldfi!(template.next());
    let template = yieldfi!(template.next_email_false());
    let template = yieldfv!(template.csrf_token(session.session().0));
    let template = yieldfi!(template.next());
    let template = yieldfi!(template.next());
    let mut template = yieldfi!(template.next());
    let mut connection = match pool.get().await {
        Ok(connection) => connection,
        Err(erroro) => {
            let template = yieldfi!(template.next_enter_loop());
            let template = yieldfi!(template.next_error_true());
            let template = yieldfv!(template.error_message(AppError::from(erroro).to_string()));
            let template = yieldfi!(template.next());
            let template = yieldfi!(template.next_end_loop());
            yieldfi!(template.next());
            return;
        }
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
            let template = yieldfi!(template.next_end_loop());
            yieldfi!(template.next());
            return; // TODO FIXME
        }
    };
    while let Some(x) = stream.next().await {
        let inner_template = yieldfi!(template.next_enter_loop());
        let inner_template = yieldfi!(inner_template.next_error_false());
        let x = x.unwrap();
        let inner_template = yieldfv!(inner_template.title(x.title));
        let inner_template = yieldfi!(inner_template.next());
        let inner_template = yieldfv!(inner_template.description(x.info));
        template = yieldfi!(inner_template.next());
    }
    let template = yieldfi!(template.next_end_loop());
    yieldfi!(template.next());
}

pub async fn list(
    pool: Pool,
    session: Session,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible>>, AppError> {
    let stream = AsyncIteratorStream(list_internal(pool, session));
    Ok(Response::builder()
        .status(StatusCode::OK)
        .typed_header(ContentType::html())
        .body(StreamBody::new(stream))
        .unwrap())
}
