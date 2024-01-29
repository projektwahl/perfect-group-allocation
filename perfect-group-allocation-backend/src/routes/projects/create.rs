use std::borrow::Cow;
use std::convert::Infallible;
use std::pin::pin;
use std::time::{SystemTime, UNIX_EPOCH};

use async_zero_cost_templating::{html, TemplateToStream};
use bytes::{Buf, Bytes};
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use headers::ContentType;
use http::header::LOCATION;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::Empty;
use perfect_group_allocation_config::Config;
use perfect_group_allocation_database::models::NewProject;
use perfect_group_allocation_database::schema::project_history;
use perfect_group_allocation_database::Pool;

use crate::components::main::main;
use crate::error::AppError;
use crate::session::{ResponseSessionExt, Session};
use crate::{either_http_body, CreateProjectPayload, CsrfSafeForm, ResponseTypedHeaderExt};

either_http_body!(boxed EitherBody 1 2);

// here we return a body that borrows the session. but the headers are already sent then to we have to implement the abstraction properly
// maybe in this case it makes sense to clone for the body but still borrow for the head so you can set cookies before
pub async fn create<'a>(
    request: hyper::Request<
        impl http_body::Body<Data = impl Buf + Send + 'static, Error = AppError> + Send + 'static,
    >,
    session: Session,
    config: Config,
    pool: Pool,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible> + Send + 'static>, AppError>
{
    let form = CsrfSafeForm::<CreateProjectPayload>::from_request(request, &session)
        .await
        .unwrap();

    let empty_title = form.value.title.is_empty();
    let empty_description = form.value.description.is_empty();

    let global_error = if !empty_title && !empty_description {
        return (|| async {
            let mut connection = pool.get().await?;
            diesel::insert_into(project_history::table)
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
                .await?;
            Ok(Response::builder()
                .with_session(session)
                .status(StatusCode::SEE_OTHER)
                .header(LOCATION, "/list")
                .body(EitherBody::Option1(Empty::new()))
                .unwrap())
        })()
        .await;
    } else {
        Ok::<(), AppError>(())
    };

    let csrf_token = session.csrf_token();
    let csrf_token = &csrf_token;

    let title_value = &form.value.title;
    let description_value = &form.value.description;

    let result = {
        let (tx_orig, rx) = tokio::sync::mpsc::channel(1);

        let tx = tx_orig.clone();

        let future = async {
            html! {
                <h1 class="center">"Create project"</h1>

                <form class="container-small" method="post" enctype="application/x-www-form-urlencoded">
                    if let Err(global_error) = global_error {
                        <div class="error-message">"Es ist ein Fehler aufgetreten: "(Cow::Owned(global_error.to_string()))</div>
                    }

                    <input type="hidden" name="csrf_token" value=[(Cow::Borrowed(csrf_token))]>

                    if empty_title {
                        <div class="error-message">"title must not be empty"</div>
                    }
                    <label for="title">"Title:"</label>
                    <input if empty_title { class="error" } id="title" name="title" type="text" value=[(Cow::Borrowed(title_value))]>

                    if empty_description {
                        <div class="error-message">"description must not be empty"</div>
                    }
                    <label for="description">"Description:"</label>
                    <input if empty_description { class="error" } id="description" name="description" type="text" value=[(Cow::Borrowed(description_value))] >

                    <button type="submit">"Create"</button>

                    <a href="/list">"Show all projects"</a>
                </form>
            }
        };
        let future = main(&tx_orig, "Create Project".into(), &session, &config, future);
        let stream = pin!(TemplateToStream::new(future, rx));
        stream.collect::<String>().await
    };

    Ok(Response::builder()
        .with_session(session)
        .status(StatusCode::OK)
        .typed_header(ContentType::html())
        .body(EitherBody::Option2(result))
        .unwrap())
}
