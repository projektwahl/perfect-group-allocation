use std::convert::Infallible;

use bytes::{Buf, Bytes};
use headers::ContentType;
use http::header::LOCATION;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::{BodyExt as _, Empty, Limited, StreamBody};
use perfect_group_allocation_config::Config;
use perfect_group_allocation_css::index_css;
use perfect_group_allocation_openidconnect::{
    finish_authentication, OpenIdRedirect, OpenIdRedirectInner,
};
use serde::{Deserialize, Serialize};
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::Unsafe;

use crate::error::AppError;
use crate::session::Session;
use crate::{either_http_body, yieldfi, yieldfv, ResponseTypedHeaderExt};

// TODO FIXME check that form does an exact check and no unused inputs are accepted

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectErrorTemplate {
    csrf_token: String,
    error: String,
    error_description: String,
}

either_http_body!(EitherBody 1 2);

pub async fn openid_redirect(
    config: Config,
    request: hyper::Request<
        impl http_body::Body<Data = impl Buf + Send, Error = hyper::Error> + Send + 'static,
    >,
    mut session: Session, // what if this here could be a reference?
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible>>, AppError> {
    let session_ref = &mut session;

    let body: Bytes = Limited::new(request.into_body(), 100)
        .collect()
        .await
        .unwrap()
        .to_bytes();

    let form: OpenIdRedirect<OpenIdRedirectInner> = serde_urlencoded::from_bytes(&body).unwrap();

    // what if privatecookiejar (and session?) would be non-owning (I don't want to clone them)
    // TODO FIXME errors also need to return the session?

    let expected_csrf_token = session_ref.session().0;

    let openid_session = session_ref.get_and_remove_openidconnect()?;

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    match form.inner {
        OpenIdRedirectInner::Error(err) => {
            let result = async gen move {
                let template = yieldfi!(crate::routes::openid_redirect());
                let template = yieldfi!(template.next());
                let template = yieldfi!(template.next());
                let template = yieldfv!(template.page_title("Create Project"));
                let template = yieldfi!(template.next());
                let template = yieldfv!(
                    template
                        .indexcss_version_unsafe(Unsafe::unsafe_input(index_css!().1.to_string()))
                );
                let template = yieldfi!(template.next());
                let template = yieldfi!(template.next());
                let template = yieldfi!(template.next_email_false());
                let template = yieldfv!(template.csrf_token(expected_csrf_token));
                let template = yieldfi!(template.next());
                let template = yieldfi!(template.next());
                let template = yieldfi!(template.next());
                let template = yieldfv!(template.error(err.error));
                let template = yieldfi!(template.next());
                let template = yieldfv!(template.error_description(err.error_description));
                let template = yieldfi!(template.next());
                yieldfi!(template.next());
            };
            let stream = AsyncIteratorStream(result);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .typed_header(ContentType::html())
                .body(EitherBody::Option1(StreamBody::new(stream)))
                .unwrap())
        }
        OpenIdRedirectInner::Success(ok) => {
            let result = finish_authentication(
                config,
                openid_session,
                OpenIdRedirect {
                    state: form.state,
                    inner: ok,
                },
            )
            .await?;

            session_ref.set_openid_session(Some(result));

            Ok(Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, "/list")
                .body(EitherBody::Option2(Empty::new()))
                .unwrap())
        }
    }
}
