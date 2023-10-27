use std::convert::Infallible;
use std::sync::Arc;
use std::task::Poll;

use axum::response::{IntoResponse, IntoResponseParts, Response};
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::PrivateCookieJar;
use futures_util::future::BoxFuture;
use oauth2::PkceCodeVerifier;
use openidconnect::Nonce;
use rand::{thread_rng, Rng};
use tokio::sync::Mutex;
use tower::{Layer, Service};

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

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
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
                session: session.clone(),
                body,
            },
        ));
        Box::pin(async move {
            let response: Response = future.await?;
            let cookies = Arc::into_inner(session).unwrap().into_inner();
            Ok((cookies, response).into_response())
        })
    }
}

#[derive(Clone)]
pub struct Session {
    private_cookies: PrivateCookieJar,
}

impl Session {
    const COOKIE_NAME_OPENID_CSRF_TOKEN: &'static str = "__Host-openid-csrf-token";
    const COOKIE_NAME_OPENID_NONCE: &'static str = "__Host-openid-nonce";
    const COOKIE_NAME_PKCE_VERIFIER: &'static str = "__Host-openid-pkce-verifier";

    #[must_use]
    pub const fn new(private_cookies: PrivateCookieJar) -> Self {
        Self { private_cookies }
    }

    pub fn session_id(&mut self) -> String {
        const COOKIE_NAME: &str = "__Host-session_id";
        if self.private_cookies.get(COOKIE_NAME).is_none() {
            let rand_string: String = thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();

            let session_id = rand_string;
            let cookie = Cookie::build(COOKIE_NAME, session_id)
                .http_only(true)
                .same_site(axum_extra::extract::cookie::SameSite::Strict)
                .secure(true)
                .finish();
            self.private_cookies = self.private_cookies.clone().add(cookie);
        }
        self.private_cookies
            .get(COOKIE_NAME)
            .map(|c| c.value().to_string())
            .unwrap()
    }

    pub fn set_openid_pkce_verifier(&mut self, verifier: &PkceCodeVerifier) {
        let cookie = Cookie::build(
            Self::COOKIE_NAME_PKCE_VERIFIER,
            verifier.secret().to_owned(),
        )
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .secure(true)
        .finish();
        self.private_cookies = self.private_cookies.clone().add(cookie);
    }

    #[must_use]
    pub fn openid_pkce_verifier(&self) -> PkceCodeVerifier {
        self.private_cookies
            .get(Self::COOKIE_NAME_PKCE_VERIFIER)
            .map(|c| PkceCodeVerifier::new(c.value().to_string()))
            .unwrap()
    }

    pub fn set_openid_nonce(&mut self, nonce: &Nonce) {
        let cookie = Cookie::build(Self::COOKIE_NAME_OPENID_NONCE, nonce.secret().to_owned())
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Strict)
            .secure(true)
            .finish();
        self.private_cookies = self.private_cookies.clone().add(cookie);
    }

    pub fn set_openid_csrf_token(&mut self, csrf_token: &oauth2::CsrfToken) {
        let cookie = Cookie::build(
            Self::COOKIE_NAME_OPENID_CSRF_TOKEN,
            csrf_token.secret().to_owned(),
        )
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .secure(true)
        .finish();
        self.private_cookies = self.private_cookies.clone().add(cookie);
    }

    #[must_use]
    pub fn openid_nonce(&self) -> Nonce {
        self.private_cookies
            .get(Self::COOKIE_NAME_OPENID_NONCE)
            .map(|c| Nonce::new(c.value().to_string()))
            .unwrap()
    }

    #[must_use]
    pub fn openid_csrf_token(&self) -> oauth2::CsrfToken {
        self.private_cookies
            .get(Self::COOKIE_NAME_OPENID_CSRF_TOKEN)
            .map(|c| oauth2::CsrfToken::new(c.value().to_string()))
            .unwrap()
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
