use std::convert::Infallible;
use std::pin::pin;

use async_zero_cost_templating::{html, TemplateToStream};
use bytes::Bytes;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use headers::ContentType;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::StreamBody;
use perfect_group_allocation_config::Config;
use perfect_group_allocation_database::models::ProjectHistoryEntry;
use perfect_group_allocation_database::schema::project_history;
use perfect_group_allocation_database::Pool;
use tracing::error;

use crate::components::main::main;
use crate::error::AppError;
use crate::routes::indexcss::INDEX_CSS_VERSION;
use crate::session::{ResponseSessionExt as _, Session};
use crate::ResponseTypedHeaderExt as _;

pub async fn list(
    session: Session,
    config: &Config,
    pool: Pool,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible> + Send + 'static>, AppError>
{
    let csrf_token = session.csrf_token();

    let result = {
        let (tx_orig, rx) = tokio::sync::mpsc::channel(1);
        let tx = tx_orig.clone();
        let future = async move {
            html!(
                <div>
                    {
                        let mut connection = match pool.get().await {
                            Ok(connection) => connection,
                            Err(erroro) => {
                                // AppError::from(erroro)
                                return;
                            }
                        };
                        let mut projects = match project_history::table
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
                                return; // TODO FIXME
                            }
                        };
                    }
                    for project in projects {
                        if error {
                            <div class="error-message">"Es ist ein Fehler aufgetreten: "(error_message)</div>
                        } else {
                            "title: "(title)<br>
                            "description: "(description)<br>
                            <br>
                        }
                    }
                </div>
            )
        };
        let future = main(tx_orig, "Projects".into(), &session, &config, future);
        let stream = pin!(TemplateToStream::new(future, rx));
        // I think we should sent it at once with a content length when it is not too large
        stream.collect::<String>().await
    };
    let result = async move {
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
