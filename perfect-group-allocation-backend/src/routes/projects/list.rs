use std::convert::Infallible;

use bytes::Bytes;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use headers::ContentType;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::StreamBody;
use perfect_group_allocation_database::models::ProjectHistoryEntry;
use perfect_group_allocation_database::schema::project_history;
use perfect_group_allocation_database::Pool;
use tracing::error;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::Unsafe;

use crate::error::AppError;
use crate::routes::indexcss::INDEX_CSS_VERSION;
use crate::routes::list_projects;
use crate::session::{ResponseSessionExt as _, Session};
use crate::{yieldfi, yieldfv, ResponseTypedHeaderExt as _};

pub async fn list(
    session: Session,
    pool: Pool,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible> + Send + 'static>, AppError>
{
    let csrf_token = session.csrf_token();
    let result = async gen move {
        let template = yieldfi!(list_projects());
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = yieldfv!(template.page_title("Projects"));
        let template = yieldfi!(template.next());
        let template = yieldfv!(
            template.indexcss_version_unsafe(Unsafe::unsafe_input(INDEX_CSS_VERSION.to_string()))
        );
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next_email_false());
        let template = yieldfv!(template.csrf_token(csrf_token));
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
    };
    let stream = AsyncIteratorStream(result);
    Ok(Response::builder()
        .with_session(session)
        .status(StatusCode::OK)
        .typed_header(ContentType::html())
        .body(StreamBody::new(stream))
        .unwrap())
}
