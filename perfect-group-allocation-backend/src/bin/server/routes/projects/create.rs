use alloc::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::response::IntoResponse;
use bytes::Bytes;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use futures_util::StreamExt;
use http::header;
use perfect_group_allocation_database::models::{NewProject, ProjectHistoryEntry};
use perfect_group_allocation_database::schema::project_history;
use perfect_group_allocation_database::DatabaseConnection;
use tracing::error;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{yieldoki, yieldokv};

use super::list::create_project;
use crate::error::AppError;
use crate::session::Session;
use crate::{CreateProjectPayload, CsrfSafeForm};

pub async fn create(
    DatabaseConnection(mut connection): DatabaseConnection,
    session: Session,
    form: CsrfSafeForm<CreateProjectPayload>,
) -> (Session, impl IntoResponse) {
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

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use axum::response::IntoResponse as _;
    use axum_extra::extract::cookie::Key;
    use axum_extra::extract::PrivateCookieJar;
    use http_body_util::BodyExt;
    use perfect_group_allocation_database::{
        get_database_connection_from_env, DatabaseConnection, DatabaseError,
    };

    use crate::error::AppError;
    use crate::session::Session;
    use crate::{create, CreateProjectPayload, CsrfSafeForm};

    #[tokio::test]
    async fn hello_world() -> Result<(), AppError> {
        let database = get_database_connection_from_env()?;
        let session = Session::new(PrivateCookieJar::new(Key::generate()));
        let form = CsrfSafeForm {
            value: CreateProjectPayload {
                csrf_token: String::new(),
                title: "test".to_owned(),
                description: "test".to_owned(),
            },
        };

        let (_session, response) = create(
            DatabaseConnection(database.get().await.map_err(DatabaseError::from)?),
            session,
            form,
        )
        .await;
        let response = response.into_response();
        let binding = response.into_body().collect().await.unwrap().to_bytes();
        let response = from_utf8(&binding).unwrap();
        assert!(response.contains(
            "Error database error: Failed to acquire connection from pool: Connection pool timed \
             out"
        ));

        Ok(())
    }
}
