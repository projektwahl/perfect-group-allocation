use alloc::borrow::Cow;

use axum::response::IntoResponse;
use bytes::Bytes;
use futures_util::StreamExt;
use http::header;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{yieldoki, yieldokv};

use crate::error::AppError;
use crate::routes::projects::list::create_project;
use crate::session::Session;

pub async fn index(session: Session) -> (Session, impl IntoResponse) {
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
        let template = yieldokv!(template.title(""));
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next_title_error_false());
        let template = yieldokv!(template.description(""));
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next_description_error_false());
        yieldoki!(template.next());
    };
    let stream =
        AsyncIteratorStream(result).map(|elem: Result<Cow<'static, str>, AppError>| match elem {
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
