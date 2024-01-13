use alloc::borrow::Cow;
use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use headers::ContentType;
use http::{header, Response, StatusCode};
use http_body::{Body, Frame};
use http_body_util::{BodyExt, Empty, StreamBody};
use perfect_group_allocation_css::index_css;
use perfect_group_allocation_database::models::NewProject;
use perfect_group_allocation_database::schema::project_history;
use perfect_group_allocation_database::{DatabaseConnection, DatabaseError, Pool};
use tracing::error;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{yieldoki, yieldokv, Unsafe};

use crate::error::AppError;
use crate::routes::create_project;
use crate::session::Session;
use crate::{
    either_http_body, yieldfi, yieldfv, CreateProjectPayload, CsrfSafeForm, ResponseTypedHeaderExt,
};

either_http_body!(EitherBody 1 2);

pub async fn create(
    request: hyper::Request<hyper::body::Incoming>,
    pool: Pool,
    session: Session, // TODO FIXME extract in here
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible>>, AppError> {
    let session_clone = session.clone();
    let form = CsrfSafeForm::<CreateProjectPayload>::from_request(request, session)
        .await
        .unwrap();

    let empty_title = form.value.title.is_empty();
    let empty_description = form.value.description.is_empty();
    let mut global_error = None;

    if !empty_title && !empty_description {
        match pool.get().await {
            Ok(mut connection) => {
                if let Err(error) = diesel::insert_into(project_history::table)
                    .values(NewProject {
                        id: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .subsec_nanos()
                            .try_into()
                            .unwrap(),
                        title: form.value.title.clone(),
                        info: form.value.description.clone(),
                    })
                    .execute(&mut connection)
                    .await
                {
                    global_error = Some(AppError::from(DatabaseError::from(error)).to_string());
                };
                return Ok(Response::builder()
                    .status(if global_error.is_some() {
                        StatusCode::INTERNAL_SERVER_ERROR
                    } else {
                        StatusCode::OK
                    })
                    .body(EitherBody::Option1(Empty::new()))
                    .unwrap());
            }
            Err(error) => {
                global_error = Some(AppError::from(error).to_string());
            }
        };
    }

    let status_code = if global_error.is_some() {
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    };

    let result = async gen move {
        let template = yieldfi!(create_project());
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = yieldfv!(template.page_title("Create Project"));
        let template = yieldfi!(template.next());
        let template = yieldfv!(
            template.indexcss_version_unsafe(Unsafe::unsafe_input(index_css!().1.to_string()))
        );
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next_email_false());
        let template = yieldfv!(template.csrf_token(session_clone.session().0));
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = if let Some(global_error) = global_error {
            let inner_template = yieldfi!(template.next_error_true());
            let inner_template = yieldfv!(inner_template.error_message(global_error));
            yieldfi!(inner_template.next())
        } else {
            yieldfi!(template.next_error_false())
        };
        let template = yieldfv!(template.csrf_token(session_clone.session().0));
        let template = yieldfi!(template.next());
        let template = if empty_title {
            let template = yieldfi!(template.next_title_error_true());
            let template = yieldfv!(template.title_error("title must not be empty"));
            yieldfi!(template.next())
        } else {
            yieldfi!(template.next_title_error_false())
        };
        let template = yieldfv!(template.title(form.value.title.clone()));
        let template = yieldfi!(template.next());
        let template = if empty_description {
            let template = yieldfi!(template.next_description_error_true());
            let template = yieldfv!(template.description_error("description must not be empty"));
            yieldfi!(template.next())
        } else {
            yieldfi!(template.next_description_error_false())
        };
        let template = yieldfv!(template.description(form.value.description.clone()));
        let template = yieldfi!(template.next());

        yieldfi!(template.next());
    };
    let stream = AsyncIteratorStream(result);
    Ok(Response::builder()
        .status(status_code)
        .typed_header(ContentType::html())
        .body(EitherBody::Option2(StreamBody::new(stream)))
        .unwrap())
}
