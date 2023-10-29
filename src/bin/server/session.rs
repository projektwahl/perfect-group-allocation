use alloc::sync::Arc;
use core::convert::Infallible;
use core::task::Poll;
use std::sync::Mutex;

use axum::extract::State;
use axum::response::{IntoResponse, IntoResponseParts, Response};
use axum::RequestPartsExt;
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::PrivateCookieJar;
use chrono::{DateTime, Utc};
use futures_util::future::BoxFuture;
use handlebars::Handlebars;
use miniserde::Deserialize;
use oauth2::{PkceCodeVerifier, RefreshToken};
use openidconnect::{EndUserEmail, Nonce};
use rand::{thread_rng, Rng};
use tower::{Layer, Service};

use crate::error::{AppError, AppErrorWithMetadata};
use crate::{BodyWithSession, MyState, HANDLEBARS};

#[derive(Clone)]
pub struct SessionLayer {
    pub key: Key,
}

impl<S> Layer<S> for SessionLayer {
    type Service = SessionMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware {
            inner,
            key: self.key.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SessionMiddleware<S> {
    inner: S,
    key: Key,
}

impl<S, ReqBody> Service<hyper::Request<ReqBody>> for SessionMiddleware<S>
where
    S: Service<
            hyper::Request<BodyWithSession<ReqBody>>,
            Response = axum::response::Response,
            Error = Infallible,
        > + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Infallible>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut core::task::Context<'_>) -> Poll<Result<(), Infallible>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: hyper::Request<ReqBody>) -> Self::Future {
        let (parts, body) = request.into_parts();
        let session = Session::new(PrivateCookieJar::from_headers(
            &parts.headers,
            self.key.clone(),
        ));
        let session = Arc::new(Mutex::new(session));
        let future = self.inner.call(hyper::Request::from_parts(
            parts,
            BodyWithSession {
                session: Arc::clone(&session),
                body,
            },
        ));
        Box::pin(async move {
            let response: Response = future.await?;
            // this may not work if you return a streaming response
            // TODO FIXME retrieve request id and csrf token from session
            match Arc::try_unwrap(session) {
                Ok(cookies) => Ok((cookies.into_inner().unwrap(), response).into_response()),
                Err(cookies) => Ok(AppErrorWithMetadata {
                    session: cookies,
                    request_id: "no-request-id".to_owned(),
                    app_error: AppError::SessionStillHeld,
                }
                .into_response()),
            }
        })
    }
}

#[derive(miniserde::Serialize)]
pub struct SessionCookieStrings {
    email: String,
    expiration: String,
    refresh_token: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct SessionCookie {
    pub email: EndUserEmail,
    pub expiration: DateTime<Utc>,
    pub refresh_token: RefreshToken,
}

fn test_to_string(value: &(String, Option<SessionCookieStrings>)) -> String {
    miniserde::json::to_string(value)
}

#[derive(Clone)]
pub struct Session {
    private_cookies: PrivateCookieJar,
}

impl Session {
    const COOKIE_NAME_OPENIDCONNECT: &'static str = "__Host-openidconnect";
    const COOKIE_NAME_SESSION: &'static str = "__Host-session";

    #[must_use]
    pub fn new(private_cookies: PrivateCookieJar) -> Self {
        let mut session = Self { private_cookies };
        if session.optional_session().is_none() {
            session.set_session(None);
        }
        session
    }

    fn optional_session(&self) -> Option<(String, Option<SessionCookie>)> {
        self.private_cookies
            .get(Self::COOKIE_NAME_SESSION)
            .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
    }

    #[must_use]
    pub fn session(&self) -> (String, Option<SessionCookie>) {
        // constructor and all method calls ensure this is not None
        #[allow(clippy::unwrap_used)]
        self.optional_session().unwrap()
    }

    pub fn set_session(&mut self, input: Option<SessionCookie>) -> (String, Option<SessionCookie>) {
        let session_id: String = thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        let value = (
            session_id.clone(),
            input.as_ref().map(|session_cookie| SessionCookieStrings {
                email: session_cookie.email.to_string(),
                expiration: session_cookie.expiration.to_string(),
                refresh_token: session_cookie.refresh_token.secret().to_string(),
            }),
        );
        let cookie = Cookie::build(Self::COOKIE_NAME_SESSION, test_to_string(&value))
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Strict)
            .secure(true)
            .finish();
        self.private_cookies = self.private_cookies.clone().add(cookie);
        (session_id, input)
    }

    pub fn set_openidconnect(
        &mut self,
        input: &(&PkceCodeVerifier, &Nonce, &oauth2::CsrfToken),
    ) -> Result<(), AppError> {
        let cookie = Cookie::build(
            Self::COOKIE_NAME_OPENIDCONNECT,
            serde_json::to_string(input)?,
        )
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax) // needed because top level callback is cross-site
        .secure(true)
        .finish();
        self.private_cookies = self.private_cookies.clone().add(cookie);
        Ok(())
    }

    pub fn get_and_remove_openidconnect(
        &mut self,
    ) -> Result<(PkceCodeVerifier, Nonce, oauth2::CsrfToken), AppError> {
        let return_value = Ok(self
            .private_cookies
            .get(Self::COOKIE_NAME_OPENIDCONNECT)
            .map(|cookie| serde_json::from_str(cookie.value()))
            .ok_or(AppError::OpenIdTokenNotFound)??);
        let cookie = Cookie::build(Self::COOKIE_NAME_OPENIDCONNECT, "")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax) // needed because top level callback is cross-site
            .secure(true)
            .finish();
        self.private_cookies = self.private_cookies.clone().remove(cookie);
        return_value
    }
}

impl IntoResponseParts for Session {
    type Error = Infallible;

    fn into_response_parts(
        self,
        res: axum::response::ResponseParts,
    ) -> Result<axum::response::ResponseParts, Self::Error> {
        self.private_cookies.into_response_parts(res)
    }
}
