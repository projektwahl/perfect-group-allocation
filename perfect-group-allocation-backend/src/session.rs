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

pub trait IntoCookieValue {
    fn into_cookie_value(self) -> Option<String>;
}

impl IntoCookieValue for String {
    fn into_cookie_value(self) -> Option<String> {
        Some(self)
    }
}

impl IntoCookieValue for () {
    fn into_cookie_value(self) -> Option<String> {
        None
    }
}

impl IntoCookieValue for Option<String> {
    fn into_cookie_value(self) -> Option<String> {
        self
    }
}

// we don't want to store cookies we don't need
// I think the csrf token needs to be signed/encrypted
pub struct Session<
    CsrfToken: IntoCookieValue = Option<String>,
    OpenIdConnectSession: IntoCookieValue = Option<String>,
    TemporaryOpenIdConnectState: IntoCookieValue = Option<String>,
> {
    /// Only static resources don't need this. All other pages need it for the login link in the header.
    // bool is true when the value was changed
    csrf_token: (CsrfToken, bool),
    openidconnect_session: (OpenIdConnectSession, bool),
    temporary_openidconnect_state: (TemporaryOpenIdConnectState, bool),
}

impl Session {
    pub fn new<T>(request: &Request<T>) -> Self {
        let mut new = Session {
            csrf_token: (None, false),
            openidconnect_session: (None, false),
            temporary_openidconnect_state: (None, false),
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
                COOKIE_NAME_CSRF_TOKEN => new.csrf_token = (Some(cookie.value().to_owned()), false),
                COOKIE_NAME_OPENIDCONNECT_SESSION => {
                    new.openidconnect_session = (Some(cookie.value().to_owned()), false)
                }
                COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE => {
                    new.temporary_openidconnect_state =
                        (Some(serde_json::from_str(cookie.value()).unwrap()), false)
                }
                _ => {
                    // ignore the cookies that are not interesting for us
                }
            });
        new
    }

    pub fn to_cookies<T>(self, response: &mut http::Response<T>) {
        if let (value, true) = self.csrf_token {
            let cookie = match value {
                Some(value) => Cookie::build((COOKIE_NAME_CSRF_TOKEN, value)).build(),
                None => Cookie::build(COOKIE_NAME_CSRF_TOKEN).build(),
            };
            response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::try_from(cookie.to_string()).unwrap(),
            );
        }
        if let (value, true) = self.openidconnect_session {
            let cookie = match value {
                Some(value) => Cookie::build((COOKIE_NAME_OPENIDCONNECT_SESSION, value)).build(),
                None => Cookie::build(COOKIE_NAME_OPENIDCONNECT_SESSION).build(),
            };
            response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::try_from(cookie.to_string()).unwrap(),
            );
        }
        if let (value, true) = self.temporary_openidconnect_state {
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

impl<OpenIdConnectSession: IntoCookieValue, TemporaryOpenIdConnectState: IntoCookieValue>
    Session<Option<String>, OpenIdConnectSession, TemporaryOpenIdConnectState>
{
    pub fn ensure_csrf_token(
        self,
    ) -> Session<String, OpenIdConnectSession, TemporaryOpenIdConnectState> {
        if let (Some(csrf_token), false) = self.csrf_token {
            Session {
                csrf_token: (csrf_token, false),
                openidconnect_session: self.openidconnect_session,
                temporary_openidconnect_state: self.temporary_openidconnect_state,
            }
        } else {
            let csrf_token: String = thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();
            Session {
                csrf_token: (csrf_token, true),
                openidconnect_session: self.openidconnect_session,
                temporary_openidconnect_state: self.temporary_openidconnect_state,
            }
        }
    }
}

impl<CsrfToken: IntoCookieValue, TemporaryOpenIdConnectState: IntoCookieValue>
    Session<CsrfToken, Option<String>, TemporaryOpenIdConnectState>
{
    pub fn with_openidconnect_session(
        self,
        input: String,
    ) -> Session<CsrfToken, String, TemporaryOpenIdConnectState> {
        self.openidconnect_session = CookieValue::Changed(Some(input));
        Session {
            csrf_token: self.csrf_token,
            openidconnect_session: self.openidconnect_session,
            temporary_openidconnect_state: self.temporary_openidconnect_state,
        }
    }

    pub fn without_openidconnect_session(
        self,
    ) -> Session<CsrfToken, (), TemporaryOpenIdConnectState> {
        if let (None, false) = self.openidconnect_session {
            Session {
                csrf_token: self.csrf_token,
                openidconnect_session: ((), false),
                temporary_openidconnect_state: self.temporary_openidconnect_state,
            }
        } else {
            Session {
                csrf_token: self.csrf_token,
                openidconnect_session: ((), true),
                temporary_openidconnect_state: self.temporary_openidconnect_state,
            }
        }
    }
}

impl<CsrfToken: IntoCookieValue, OpenIdConnectSession: IntoCookieValue>
    Session<CsrfToken, OpenIdConnectSession, Option<String>>
{
    pub fn with_temporary_openidconnect_state(
        self,
        input: &OpenIdSession,
    ) -> Session<CsrfToken, OpenIdConnectSession, String> {
        Session {
            csrf_token: self.csrf_token,
            openidconnect_session: self.openidconnect_session,
            temporary_openidconnect_state: (Some(serde_json::to_string(input).unwrap()), true),
        }
    }

    pub fn get_and_remove_temporary_openidconnect_state(
        self,
    ) -> Result<(String, Session<CsrfToken, OpenIdConnectSession, ()>), AppError> {
        if let (Some(temporary_openidconnect_state), false) = self.temporary_openidconnect_state {
            Ok((
                temporary_openidconnect_state,
                Session {
                    csrf_token: self.csrf_token,
                    openidconnect_session: self.openidconnect_session,
                    temporary_openidconnect_state: (None, true),
                },
            ))
        } else {
            Err(AppError::OpenIdTokenNotFound)
        }
    }
}
