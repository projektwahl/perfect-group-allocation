use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};

use async_zero_cost_templating::{html, FutureToStream};
use bytes::{Buf, Bytes};
use diesel_async::RunQueryDsl;
use headers::ContentType;
use http::header::LOCATION;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::{Empty, StreamBody};
use perfect_group_allocation_config::Config;
use perfect_group_allocation_database::models::NewProject;
use perfect_group_allocation_database::schema::project_history;
use perfect_group_allocation_database::Pool;

use crate::components::main::main;
use crate::error::AppError;
use crate::routes::indexcss::INDEX_CSS_VERSION;
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

    let abc = |stream: FutureToStream| async {
        let csrf_token = session.csrf_token();

        let html = html! {
            <h1 class="center">"Create project"</h1>

            <form class="container-small" method="post" enctype="application/x-www-form-urlencoded">
                if error {
                    <div class="error-message">"Es ist ein Fehler aufgetreten: "(error_message)</div>
                }

                <input type="hidden" name="csrf_token" value="{{csrf_token}}">

                if title_error {
                    <div class="error-message">(title_error)</div>
                }
                <label for="title">"Title:"</label>
                <input if title_error { class="error" } id="title" name="title" type="text" if title { value=[(title)] } >

                if description_error {
                    <div class="error-message">(description_error)</div>
                }
                <label for="description">"Description:"</label>
                <input if description_error { class="error" } id="description" name="description" type="text" if description { value=[(description)] } >

                <button type="submit">"Create"</button>

                <a href="/list">"Show all projects"</a>
            </form>
        };
        main(stream, "Create Project".into(), session, config, html)
    };

    let result = async move {
        let template = if let Err(global_error) = global_error {
            let inner_template = yieldfi!(template.next_error_true());
            let inner_template = yieldfv!(inner_template.error_message(global_error.to_string()));
            yieldfi!(inner_template.next())
        } else {
            yieldfi!(template.next_error_false())
        };
        let template = yieldfv!(template.csrf_token(csrf_token));
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
        .with_session(session)
        .status(StatusCode::OK)
        .typed_header(ContentType::html())
        .body(EitherBody::Option2(StreamBody::new(stream)))
        .unwrap())
}
