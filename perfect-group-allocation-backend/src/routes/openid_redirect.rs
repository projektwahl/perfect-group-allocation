use alloc::borrow::Cow;

use anyhow::anyhow;
use bytes::Bytes;
use futures_util::StreamExt;
use http::header;
use http_body::Body;
use perfect_group_allocation_openidconnect::{
    finish_authentication, OpenIdRedirect, OpenIdRedirectInner,
};
use serde::{Deserialize, Serialize};
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::{yieldoki, yieldokv};

use crate::error::AppError;
use crate::session::{Session, SessionCookie};
use crate::{yieldfi, yieldfv};

// TODO FIXME check that form does an exact check and no unused inputs are accepted

#[derive(Deserialize, Serialize)]
pub struct OpenIdRedirectErrorTemplate {
    csrf_token: String,
    error: String,
    error_description: String,
}

#[expect(
    clippy::disallowed_types,
    reason = "csrf protection done here explicitly"
)]
pub async fn openid_redirect(
    mut session: Session, // what if this here could be a reference?
    form: axum::Form<OpenIdRedirect>,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = AppError>>, AppError> {
    let session_ref = &mut session;
    // what if privatecookiejar (and session?) would be non-owning (I don't want to clone them)
    // TODO FIXME errors also need to return the session?

    let expected_csrf_token = session_ref.session().0;

    let openid_session = session_ref.get_and_remove_openidconnect()?;

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    match form.0.inner {
        OpenIdRedirectInner::Error(err) => {
            let result = async gen move {
                let template = yieldfi!(crate::routes::openid_redirect());
                let template = yieldfi!(template.next());
                let template = yieldfi!(template.next());
                let template = yieldfv!(template.page_title("Create Project"));
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
            let stream =
                AsyncIteratorStream(result).map(|elem: Result<Cow<'static, str>, AppError>| {
                    match elem {
                        Err(app_error) => Ok::<Bytes, AppError>(Bytes::from(format!(
                            // TODO FIXME use template here
                            "<h1>Error {}</h1>",
                            &app_error.to_string()
                        ))),
                        Ok(Cow::Owned(ok)) => Ok::<Bytes, AppError>(Bytes::from(ok)),
                        Ok(Cow::Borrowed(ok)) => Ok::<Bytes, AppError>(Bytes::from(ok)),
                    }
                });
            Ok(([(header::CONTENT_TYPE, "text/html")], stream).into_response())
        }
        OpenIdRedirectInner::Success(ok) => {
            let result = finish_authentication(
                openid_session,
                OpenIdRedirect {
                    state: form.0.state,
                    inner: ok,
                },
            )
            .await?;

            session_ref.set_session(Some(SessionCookie {
                email: email.to_owned(),
                expiration: claims.expiration(),
                refresh_token: refresh_token.to_owned(),
            }));

            Ok(Redirect::to("/list").into_response())
        }
    }
}
