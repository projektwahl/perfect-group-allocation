use alloc::borrow::Cow;
use std::convert::Infallible;

use bytes::Bytes;
use futures_util::StreamExt;
use http::{header, Response, StatusCode};
use http_body::{Body, Frame};
use http_body_util::{Full, StreamBody};
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{yieldoki, yieldokv};

use crate::error::AppError;
use crate::routes::create_project;
use crate::session::Session;

pub async fn index(
    request: hyper::Request<impl hyper::body::Body>,
    session: Session,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = AppError>>, AppError> {
    let result = async gen move {
        let template = yieldoki!(create_project());
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldokv!(template.page_title("Create Project"));
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next_email_false());
        let template = yieldokv!(template.csrf_token(session.session().0));
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldokv!(template.csrf_token(session.session().0));
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
