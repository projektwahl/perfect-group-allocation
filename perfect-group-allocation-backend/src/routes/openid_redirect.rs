use std::borrow::Cow;
use std::convert::Infallible;
use std::pin::pin;

use async_zero_cost_templating::{html, TemplateToStream};
use bytes::{Buf, Bytes};
use futures_util::StreamExt as _;
use headers::ContentType;
use http::header::LOCATION;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::{Empty, StreamBody};
use perfect_group_allocation_config::Config;
use perfect_group_allocation_openidconnect::{
    finish_authentication, OpenIdRedirect, OpenIdRedirectInner,
};
use serde::{Deserialize, Serialize};

use crate::components::main::main;
use crate::error::AppError;
use crate::routes::indexcss::INDEX_CSS_VERSION;
use crate::session::{ResponseSessionExt as _, Session};
use crate::{either_http_body, ResponseTypedHeaderExt};

// TODO FIXME check that form does an exact check and no unused inputs are accepted

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectErrorTemplate {
    csrf_token: String,
    error: String,
    error_description: String,
}

either_http_body!(either EitherBody 1 2);

pub async fn openid_redirect(
    request: hyper::Request<
        impl http_body::Body<Data = impl Buf + Send, Error = AppError> + Send + 'static,
    >,
    session: Session,
    config: &Config,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible> + Send + 'static>, AppError>
{
    let body = request.uri().query().unwrap();

    // TODO FIXME unwrap
    let form: OpenIdRedirect<OpenIdRedirectInner> = serde_urlencoded::from_str(body).unwrap();

    // what if privatecookiejar (and session?) would be non-owning (I don't want to clone them)
    // TODO FIXME errors also need to return the session?

    let expected_csrf_token = session.csrf_token();

    let (openid_session, session) = session.get_and_remove_temporary_openidconnect_state()?;

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    match form.inner {
        OpenIdRedirectInner::Error(err) => {
            let result = {
                let (tx_orig, rx) = tokio::sync::mpsc::channel(1);
                let tx = tx_orig.clone();
                let future = async move {
                    html! {
                        <h1 class="center">"OpenID Redirect"</h1>

                        <h2>"Error: "(Cow::Owned(err.error))</h2>
                        <span>"Error details: "(Cow::Owned(err.error_description))</span>
                    }
                };
                let future = main(
                    tx_orig,
                    "OpenID Redirect Error".into(),
                    &session,
                    &config,
                    future,
                );
                let stream = pin!(TemplateToStream::new(future, rx));
                // I think we should sent it at once with a content length when it is not too large
                stream.collect::<String>().await
            };

            Ok(Response::builder()
                .with_session(session)
                .status(StatusCode::OK)
                .typed_header(ContentType::html())
                .body(EitherBody::Option1(result))
                .unwrap())
        }
        OpenIdRedirectInner::Success(ok) => {
            let result = finish_authentication(
                &config,
                openid_session,
                OpenIdRedirect {
                    state: form.state,
                    inner: ok,
                },
            )
            .await?;

            let session = session.with_openidconnect_session(result);

            Ok(Response::builder()
                .with_session(session)
                .status(StatusCode::SEE_OTHER)
                .header(LOCATION, "/list")
                .body(EitherBody::Option2(Empty::new()))
                .unwrap())
        }
    }
}
