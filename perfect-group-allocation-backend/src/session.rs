use cookie::Cookie;
use http::header::{COOKIE, SET_COOKIE};
use http::{HeaderValue, Request};
use perfect_group_allocation_openidconnect::OpenIdSession;
use rand::{thread_rng, Rng as _};
use tracing::debug;

use crate::error::AppError;

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

impl IntoCookieValue for OpenIdSession {
    fn into_cookie_value(self) -> Option<String> {
        Some(serde_json::to_string(&self).unwrap())
    }
}

impl IntoCookieValue for Option<OpenIdSession> {
    fn into_cookie_value(self) -> Option<String> {
        self.map(|this| serde_json::to_string(&this).unwrap())
    }
}

// I think the csrf token needs to be signed/encrypted
/// we don't want to store cookies we don't need
#[derive(Clone)]
#[must_use]
pub struct Session<
    OpenIdConnectSession: IntoCookieValue + Clone = Option<String>,
    TemporaryOpenIdConnectState: IntoCookieValue = Option<OpenIdSession>,
> {
    /// Only static resources don't need this. All other pages need it for the login link in the header.
    // bool is true when the value was changed
    csrf_token: (String, bool),
    openidconnect_session: (OpenIdConnectSession, bool),
    temporary_openidconnect_state: (TemporaryOpenIdConnectState, bool),
}

impl<OpenIdConnectSession: IntoCookieValue + Clone, TemporaryOpenIdConnectState: IntoCookieValue>
    Session<OpenIdConnectSession, TemporaryOpenIdConnectState>
{
    pub fn csrf_token(&self) -> String {
        self.csrf_token.0.clone()
    }

    pub fn openidconnect_session(&self) -> OpenIdConnectSession {
        self.openidconnect_session.0.clone()
    }
}

pub trait ResponseSessionExt {
    #[must_use]
    fn with_session<
        OpenIdConnectSession: IntoCookieValue + Clone,
        TemporaryOpenIdConnectState: IntoCookieValue,
    >(
        self,
        session: Session<OpenIdConnectSession, TemporaryOpenIdConnectState>,
    ) -> Self;
}

impl ResponseSessionExt for hyper::http::response::Builder {
    fn with_session<
        OpenIdConnectSession: IntoCookieValue + Clone,
        TemporaryOpenIdConnectState: IntoCookieValue,
    >(
        self,
        session: Session<OpenIdConnectSession, TemporaryOpenIdConnectState>,
    ) -> Self {
        let mut this = self;
        if let (value, true) = session.csrf_token {
            let cookie = Cookie::build((COOKIE_NAME_CSRF_TOKEN, value)).build();
            this = this.header(
                SET_COOKIE,
                HeaderValue::try_from(cookie.to_string()).unwrap(),
            );
        }
        if let (value, true) = session.openidconnect_session {
            let cookie = value.into_cookie_value().map_or_else(
                || Cookie::build(COOKIE_NAME_OPENIDCONNECT_SESSION).build(),
                |value| Cookie::build((COOKIE_NAME_OPENIDCONNECT_SESSION, value)).build(),
            );
            this = this.header(
                SET_COOKIE,
                HeaderValue::try_from(cookie.to_string()).unwrap(),
            );
        }
        if let (value, true) = session.temporary_openidconnect_state {
            let cookie = value.into_cookie_value().map_or_else(
                || Cookie::build(COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE).build(),
                |value| Cookie::build((COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE, value)).build(),
            );
            this = this.header(
                SET_COOKIE,
                HeaderValue::try_from(cookie.to_string()).unwrap(),
            );
        }
        this
    }
}

impl Session {
    pub fn new<T>(request: &Request<T>) -> Self {
        let mut csrf_token = None;
        let mut openidconnect_session = None;
        let mut temporary_openidconnect_state = None;
        request
            .headers()
            .get_all(COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok())
            .map(std::borrow::ToOwned::to_owned)
            .flat_map(Cookie::split_parse)
            .filter_map(std::result::Result::ok)
            .for_each(|cookie| match cookie.name() {
                COOKIE_NAME_CSRF_TOKEN => csrf_token = Some((cookie.value().to_owned(), false)),
                COOKIE_NAME_OPENIDCONNECT_SESSION => {
                    openidconnect_session = Some(cookie.value().to_owned());
                }
                COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE => {
                    if let Ok(cookie) = serde_json::from_str(cookie.value()) {
                        temporary_openidconnect_state = cookie;
                    } else {
                        debug!("failed to parse {}", cookie.value());
                    }
                }
                _ => {
                    // ignore the cookies that are not interesting for us
                }
            });
        let csrf_token = csrf_token.unwrap_or_else(|| {
            (
                thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(30)
                    .map(char::from)
                    .collect(),
                true,
            )
        });
        Self {
            csrf_token,
            openidconnect_session: (openidconnect_session, false),
            temporary_openidconnect_state: (temporary_openidconnect_state, false),
        }
    }
}

impl<TemporaryOpenIdConnectState: IntoCookieValue>
    Session<Option<String>, TemporaryOpenIdConnectState>
{
    pub fn with_openidconnect_session(
        self,
        input: String,
    ) -> Session<String, TemporaryOpenIdConnectState> {
        Session {
            csrf_token: self.csrf_token,
            openidconnect_session: (input, true),
            temporary_openidconnect_state: self.temporary_openidconnect_state,
        }
    }

    pub fn without_openidconnect_session(self) -> Session<(), TemporaryOpenIdConnectState> {
        if self.openidconnect_session == (None, false) {
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

impl<OpenIdConnectSession: IntoCookieValue + Clone>
    Session<OpenIdConnectSession, Option<OpenIdSession>>
{
    pub fn with_temporary_openidconnect_state(
        self,
        input: OpenIdSession,
    ) -> Session<OpenIdConnectSession, OpenIdSession> {
        Session {
            csrf_token: self.csrf_token,
            openidconnect_session: self.openidconnect_session,
            temporary_openidconnect_state: (input, true),
        }
    }

    pub fn without_temporary_openidconnect_state(&self) -> Session<OpenIdConnectSession, ()> {
        Session {
            csrf_token: self.csrf_token.clone(),
            openidconnect_session: self.openidconnect_session.clone(),
            temporary_openidconnect_state: ((), false),
        }
    }

    pub fn get_and_remove_temporary_openidconnect_state(
        self,
    ) -> Result<(OpenIdSession, Session<OpenIdConnectSession, ()>), AppError> {
        if let (Some(temporary_openidconnect_state), false) = self.temporary_openidconnect_state {
            Ok((
                temporary_openidconnect_state,
                Session {
                    csrf_token: self.csrf_token,
                    openidconnect_session: self.openidconnect_session,
                    temporary_openidconnect_state: ((), true),
                },
            ))
        } else {
            Err(AppError::OpenIdTokenNotFound)
        }
    }
}
