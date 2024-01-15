use std::convert::Infallible;

use bytes::{Buf, Bytes};
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::StreamBody;
use perfect_group_allocation_css::index_css;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::Unsafe;

use crate::error::AppError;
use crate::routes::create_project;
use crate::session::Session;
use crate::{get_session, yieldfi, yieldfv};

pub async fn index(
    request: hyper::Request<
        impl http_body::Body<Data = impl Buf + Send, Error = AppError> + Send + 'static,
    >,
    session: &mut Session,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible>>, AppError> {
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
        let template = yieldfv!(template.csrf_token(session.session().0));
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next_error_false());
        let template = yieldfv!(template.csrf_token(session.session().0));
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next_title_error_false());
        let template = yieldfv!(template.title(""));
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next_description_error_false());
        let template = yieldfv!(template.description(""));
        let template = yieldfi!(template.next());
        yieldfi!(template.next());
    };
    let stream = AsyncIteratorStream(result);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(StreamBody::new(stream))
        .unwrap())
}
