use alloc::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use http::{header, Response, StatusCode};
use http_body::{Body, Frame};
use http_body_util::StreamBody;
use perfect_group_allocation_database::models::NewProject;
use perfect_group_allocation_database::schema::project_history;
use perfect_group_allocation_database::DatabaseConnection;
use tracing::error;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{yieldoki, yieldokv};

use crate::error::AppError;
use crate::routes::create_project;
use crate::session::Session;
use crate::{CreateProjectPayload, CsrfSafeForm};

pub async fn create(
    DatabaseConnection(mut connection): DatabaseConnection,
    session: Session,
    form: CsrfSafeForm<CreateProjectPayload>,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = AppError>>, AppError> {
    let session_clone = session.clone();
    let result = async gen move {
        let template = yieldoki!(create_project());
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldokv!(template.page_title("Create Project"));
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next_email_false());
        let template = yieldokv!(template.csrf_token(session_clone.session().0));
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldokv!(template.csrf_token(session_clone.session().0));
        let template = yieldoki!(template.next());
        let template = yieldokv!(template.title(form.value.title.clone()));
        let template = yieldoki!(template.next());
        let empty_title = form.value.title.is_empty();
        let template = if empty_title {
            let template = yieldoki!(template.next_title_error_true());
            let template = yieldokv!(template.title_error("title must not be empty"));
            yieldoki!(template.next())
        } else {
            yieldoki!(template.next_title_error_false())
        };
        let template = yieldokv!(template.description(form.value.description.clone()));
        let template = yieldoki!(template.next());
        let empty_description = form.value.description.is_empty();
        let template = if empty_description {
            let template = yieldoki!(template.next_description_error_true());
            let template = yieldokv!(template.description_error("description must not be empty"));
            yieldoki!(template.next())
        } else {
            yieldoki!(template.next_description_error_false())
        };
        yieldoki!(template.next());

        if empty_title || empty_description {
            return;
        }

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
            error!("{:?}", error);
            yield Ok::<Cow<'static, str>, AppError>("TODO FIXME database error".into());
        };

        // we can't stream the response and then redirect so probably add a button or so and use javascript? or maybe don't stream this page?
        //Ok(Redirect::to("/list").into_response())
    };
    let stream = AsyncIteratorStream(result).map(|elem| match elem {
        Err(app_error) => Ok(Frame::data(Bytes::from(format!(
            // TODO FIXME use template here
            "<h1>Error {}</h1>",
            &app_error.to_string()
        )))),
        Ok(Cow::Owned(ok)) => Ok(Frame::data(Bytes::from(ok))),
        Ok(Cow::Borrowed(ok)) => Ok(Frame::data(Bytes::from(ok))),
    });
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(StreamBody::new(stream))
        .unwrap())
}
