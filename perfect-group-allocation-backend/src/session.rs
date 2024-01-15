use std::marker::PhantomData;

use cookie::{Cookie, CookieJar, SameSite};
use http::header::{COOKIE, SET_COOKIE};
use http::{HeaderValue, Request, Response};
use perfect_group_allocation_openidconnect::OpenIdSession;
use rand::{thread_rng, Rng as _};

use crate::error::AppError;
use crate::routes::OpenidRedirectTemplate0;

const COOKIE_NAME_CSRF_TOKEN: &str = "__Host_csrf_token";
const COOKIE_NAME_OPENIDCONNECT_SESSION: &str = "__Host_openidconnect_session";
const COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE: &str = "__Host_temporary_openidconnect_state";

pub enum CookieValue<T> {
    Unchanged(T),
    Changed(T),
}

pub struct SessionMutableInner {
    /// Only static resources don't need this. All other pages need it for the login link in the header.
    pub csrf_token: CookieValue<Option<String>>,
    pub openidconnect_session: CookieValue<Option<String>>,
    pub temporary_openidconnect_state: CookieValue<Option<String>>,
}

impl SessionMutableInner {
    pub fn new<T>(request: &Request<T>) -> Self {
        let mut new = SessionMutableInner {
            csrf_token: CookieValue::Unchanged(None),
            openidconnect_session: CookieValue::Unchanged(None),
            temporary_openidconnect_state: CookieValue::Unchanged(None),
        };
        request
            .headers()
            .get_all(COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok())
            .map(std::borrow::ToOwned::to_owned)
            .flat_map(Cookie::split_parse)
            .filter_map(std::result::Result::ok)
            .for_each(|cookie| match cookie.name() {
                COOKIE_NAME_CSRF_TOKEN => {
                    new.csrf_token = CookieValue::Unchanged(Some(cookie.value().to_owned()))
                }
                COOKIE_NAME_OPENIDCONNECT_SESSION => {
                    new.openidconnect_session =
                        CookieValue::Unchanged(Some(cookie.value().to_owned()))
                }
                COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE => {
                    new.temporary_openidconnect_state =
                        CookieValue::Unchanged(Some(serde_json::from_str(cookie.value()).unwrap()))
                }
                _ => {
                    // ignore the cookies that are not interesting for us
                }
            });
        new
    }
}

// we don't want to store cookies we don't need
#[must_use]
pub struct Session<
    'a,
    CsrfToken = Option<String>,
    OpenIdConnectSession = Option<String>,
    TemporaryOpenIdConnectState = Option<String>,
> {
    pub inner: &'a mut SessionMutableInner,
    phantom_data: PhantomData<(CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState)>,
}

impl<'a> Session<'a> {
    pub fn new(inner: &mut SessionMutableInner) -> Self {
        assert!(matches!(inner.csrf_token, CookieValue::Unchanged(_)));
        assert!(matches!(
            inner.openidconnect_session,
            CookieValue::Unchanged(_)
        ));
        assert!(matches!(
            inner.temporary_openidconnect_state,
            CookieValue::Unchanged(_)
        ));
        Self {
            inner,
            phantom_data: PhantomData,
        }
    }
}

impl<'a, OpenIdConnectSession, TemporaryOpenIdConnectState>
    Session<'a, Option<String>, OpenIdConnectSession, TemporaryOpenIdConnectState>
{
    pub fn ensure_csrf_token(
        self,
    ) -> Session<'a, String, OpenIdConnectSession, TemporaryOpenIdConnectState> {
        if let CookieValue::Unchanged(Some(csrf_token)) = self.inner.csrf_token {
        } else {
            let csrf_token: String = thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();
            self.inner.csrf_token = CookieValue::Changed(Some(csrf_token));
        }
        Session {
            inner: self.inner,
            phantom_data: PhantomData,
        }
    }
}

impl<'a, CsrfToken, TemporaryOpenIdConnectState>
    Session<'a, CsrfToken, Option<String>, TemporaryOpenIdConnectState>
{
    pub fn with_openidconnect_session(
        self,
        input: String,
    ) -> Session<'a, CsrfToken, String, TemporaryOpenIdConnectState> {
        self.inner.openidconnect_session = CookieValue::Changed(Some(input));
        Session {
            inner: self.inner,
            phantom_data: PhantomData,
        }
    }

    pub fn without_openidconnect_session(
        &mut self,
    ) -> Session<'a, CsrfToken, (), TemporaryOpenIdConnectState> {
        if let CookieValue::Unchanged(None) = self.inner.openidconnect_session {
        } else {
            self.inner.openidconnect_session = CookieValue::Changed(None);
        }
        Session {
            inner: self.inner,
            phantom_data: PhantomData,
        }
    }
}

impl<'a, CsrfToken, OpenIdConnectSession>
    Session<'a, CsrfToken, OpenIdConnectSession, Option<String>>
{
    pub fn with_temporary_openidconnect_state(
        &mut self,
        input: &OpenIdSession,
    ) -> Session<'a, CsrfToken, OpenIdConnectSession, String> {
        self.inner.temporary_openidconnect_state =
            CookieValue::Changed(Some(serde_json::to_string(input).unwrap()));
        Session {
            inner: self.inner,
            phantom_data: PhantomData,
        }
    }

    pub fn get_and_remove_temporary_openidconnect_state(
        self,
    ) -> Result<Session<'a, CsrfToken, OpenIdConnectSession, ()>, AppError> {
        if let CookieValue::Unchanged(Some(temporary_openidconnect_state)) =
            self.inner.temporary_openidconnect_state
        {
            self.inner.temporary_openidconnect_state = CookieValue::Changed(None);
            Ok(Session {
                inner: self.inner,
                phantom_data: PhantomData,
            })
        } else {
            Err(AppError::OpenIdTokenNotFound)
        }
    }
}

// I think the csrf token needs to be signed/encrypted
impl<'a, CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState>
    Session<'a, CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState>
{
    pub fn to_cookies<T>(self, response: &mut http::Response<T>) {
        if let CookieValue::Changed(value) = self.inner.csrf_token {
            let cookie = match value {
                Some(value) => Cookie::build((COOKIE_NAME_CSRF_TOKEN, value)).build(),
                None => Cookie::build(COOKIE_NAME_CSRF_TOKEN).build(),
            };
            response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::try_from(cookie.to_string()).unwrap(),
            );
        }
        if let CookieValue::Changed(value) = self.inner.openidconnect_session {
            let cookie = match value {
                Some(value) => Cookie::build((COOKIE_NAME_OPENIDCONNECT_SESSION, value)).build(),
                None => Cookie::build(COOKIE_NAME_OPENIDCONNECT_SESSION).build(),
            };
            response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::try_from(cookie.to_string()).unwrap(),
            );
        }
        if let CookieValue::Changed(value) = self.inner.temporary_openidconnect_state {
            let cookie = match value {
                Some(value) => {
                    Cookie::build((COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE, value)).build()
                }
                None => Cookie::build(COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE).build(),
            };
            response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::try_from(cookie.to_string()).unwrap(),
            );
        }
    }
}
