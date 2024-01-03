use alloc::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::response::IntoResponse;
use bytes::Bytes;
use futures_util::StreamExt;
use http::header;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{yieldoki, yieldokv};

use super::list::create_project;
use crate::entities::project_history;
use crate::error::AppError;
use crate::session::Session;
use crate::{CreateProjectPayload, CsrfSafeForm};

pub async fn create(
    State(db): State<DatabaseConnection>,
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

        let project = project_history::ActiveModel {
            id: ActiveValue::Set(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos()
                    .try_into()
                    .unwrap(),
            ), // TODO FIXME
            title: ActiveValue::Set(form.value.title.clone()),
            description: ActiveValue::Set(form.value.description.clone()),
            ..Default::default()
        };
        if let Err(err) = project_history::Entity::insert(project).exec(&db).await {
            yield Err::<Cow<'static, str>, AppError>(err.into());
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

    use crate::database::{get_offline_test_database, get_test_database};
    use crate::error::AppError;
    use crate::session::Session;
    use crate::{create, CreateProjectPayload, CsrfSafeForm};

    #[tokio::test]
    async fn hello_world() -> Result<(), AppError> {
        let database = get_offline_test_database().await?;
        let session = Session::new(PrivateCookieJar::new(Key::generate()));
        let form = CsrfSafeForm {
            value: CreateProjectPayload {
                csrf_token: String::new(),
                title: "test".to_owned(),
                description: "test".to_owned(),
            },
        };

        let state = axum::extract::State(database);
        let (_session, response) = create(state, session, form).await;
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
