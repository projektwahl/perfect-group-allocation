use alloc::sync::Arc;
use core::convert::Infallible;
use core::task::Poll;

use axum::response::{IntoResponse, IntoResponseParts, Response};
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::PrivateCookieJar;
use futures_util::future::BoxFuture;
use oauth2::PkceCodeVerifier;
use openidconnect::Nonce;
use rand::{thread_rng, Rng};
use tokio::sync::Mutex;
use tower::{Layer, Service};

use crate::error::AppError;
use crate::BodyWithSession;

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
    S: Service<hyper::Request<BodyWithSession<ReqBody>>, Response = axum::response::Response>
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut core::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
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
            let cookies = Arc::into_inner(session)
                .expect(
                    "you still seem to be holding onto the request session somewhere. maybe you \
                     keep it inside a streaming response?",
                )
                .into_inner();
            Ok((cookies, response).into_response())
        })
    }
}

#[derive(Clone)]
pub struct Session {
    private_cookies: PrivateCookieJar,
}

impl Session {
    const COOKIE_NAME_OPENIDCONNECT: &'static str = "__Host-openidconnect";

    #[must_use]
    pub const fn new(private_cookies: PrivateCookieJar) -> Self {
        Self { private_cookies }
    }

    pub fn session_id(&mut self) -> String {
        const COOKIE_NAME: &str = "__Host-session_id";
        if let Some(cookie) = self.private_cookies.get(COOKIE_NAME) {
            cookie.value().to_owned()
        } else {
            let rand_string: String = thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();

            let session_id = rand_string;
            let cookie = Cookie::build(COOKIE_NAME, session_id.clone())
                .http_only(true)
                .same_site(axum_extra::extract::cookie::SameSite::Strict)
                .secure(true)
                .finish();
            self.private_cookies = self.private_cookies.clone().add(cookie);
            session_id
        }
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

    #[must_use]
    pub fn get_openidconnect(
        &self,
    ) -> Result<(PkceCodeVerifier, Nonce, oauth2::CsrfToken), AppError> {
        Ok(self
            .private_cookies
            .get(Self::COOKIE_NAME_OPENIDCONNECT)
            .map(|cookie| serde_json::from_str(cookie.value()))
            .ok_or(AppError::OpenIdTokenNotFound)??)
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
