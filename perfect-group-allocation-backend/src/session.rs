use std::marker::PhantomData;

use cookie::{Cookie, CookieJar, SameSite};
use http::header::COOKIE;
use http::Request;
use perfect_group_allocation_openidconnect::OpenIdSession;
use rand::{thread_rng, Rng as _};

use crate::error::AppError;
use crate::routes::OpenidRedirectTemplate0;

const COOKIE_NAME_CSRF_TOKEN: &str = "__Host_csrf_token";
const COOKIE_NAME_OPENIDCONNECT_SESSION: &str = "__Host_openidconnect_session";
const COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE: &str = "__Host_temporary_openidconnect_state";

trait Cookiey {
    fn get_value(&self) -> Option<String>;
}

trait CookieyChanged {
    fn is_changed(&self) -> bool;
}

impl Cookiey for () {
    fn get_value(&self) -> Option<String> {
        None
    }
}

impl Cookiey for String {
    fn get_value(&self) -> Option<String> {
        Some(self)
    }
}

impl<T: Cookiey> Cookiey for Option<T> {
    fn get_value(&self) -> Option<String> {
        self.map(|v| v.get_value())
    }
}

pub struct Changed<T>(T);

impl<T: Cookiey> Cookiey for Changed<T> {
    fn get_value(&self) -> Option<String> {
        Some(self.0.get_value())
    }
}

impl<T> CookieyChanged for Changed<T> {
    fn is_changed(&self) -> bool {
        true
    }
}

pub struct Unchanged<T>(T);

impl<T: Cookiey> Cookiey for Unchanged<T> {
    fn get_value(&self) -> Option<String> {
        Some(self.0.get_value())
    }
}

impl<T> CookieyChanged for Unchanged<T> {
    fn is_changed(&self) -> bool {
        false
    }
}

pub enum CookieValue<T> {
    Unchanged(Unchanged<T>),
    Changed(Changed<T>),
}

impl<T: Cookiey> Cookiey for CookieValue<T> {
    fn get_value(&self) -> Option<String> {
        match self {
            CookieValue::Unchanged(value) => value.get_value(),
            CookieValue::Changed(value) => value.get_value(),
        }
    }
}

impl<T> CookieyChanged for CookieValue<T> {
    fn is_changed(&self) -> bool {
        match self {
            CookieValue::Unchanged(_) => false,
            CookieValue::Changed(_) => true,
        }
    }
}

// we don't want to store cookies we don't need
#[must_use]
pub struct Session<
    CsrfToken: Cookiey + CookieyChanged,
    OpenIdConnectSession: Cookiey + CookieyChanged,
    TemporaryOpenIdConnectState: Cookiey + CookieyChanged,
> {
    pub csrf_token: CsrfToken,
    pub openidconnect_session: OpenIdConnectSession,
    pub temporary_openidconnect_state: TemporaryOpenIdConnectState,
}

impl Session<Unchanged<Option<String>>, Unchanged<Option<String>>, Unchanged<Option<String>>> {
    pub fn new<T>(request: Request<T>) -> Self {
        let mut new = Self {
            csrf_token: Unchanged(None),
            openidconnect_session: Unchanged(None),
            temporary_openidconnect_state: Unchanged(None),
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
                    new.csrf_token = Unchanged(Some(cookie.value().to_owned()))
                }
                COOKIE_NAME_OPENIDCONNECT_SESSION => {
                    new.openidconnect_session = Unchanged(Some(cookie.value().to_owned()))
                }
                COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE => {
                    new.temporary_openidconnect_state =
                        Unchanged(Some(serde_json::from_str(cookie.value()).unwrap()))
                }
                _ => {
                    // ignore the cookies that are not interesting for us
                }
            });
        new
    }
}

impl<
    CsrfToken: Cookiey + CookieyChanged,
    OpenIdConnectSession: Cookiey + CookieyChanged,
    TemporaryOpenIdConnectState: Cookiey + CookieyChanged,
> Session<CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState>
{
    pub fn with_csrf_token(
        self,
    ) -> Session<CookieValue<String>, OpenIdConnectSession, TemporaryOpenIdConnectState> {
        if let Unchanged(Some(csrf_token)) = self.csrf_token {
            Session {
                csrf_token: CookieValue::Unchanged(Unchanged(csrf_token)),
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
                csrf_token: CookieValue::Changed(Changed(csrf_token)),
                openidconnect_session: self.openidconnect_session,
                temporary_openidconnect_state: self.temporary_openidconnect_state,
            }
        }
    }
}

impl<
    CsrfToken: Cookiey + CookieyChanged,
    OpenIdConnectSession: Cookiey + CookieyChanged,
    TemporaryOpenIdConnectState: Cookiey + CookieyChanged,
> Session<CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState>
{
    pub fn with_openidconnect_session(
        self,
        input: String,
    ) -> Session<CsrfToken, Changed<String>, TemporaryOpenIdConnectState> {
        Session {
            csrf_token: self.csrf_token,
            openidconnect_session: Changed(input),
            temporary_openidconnect_state: self.temporary_openidconnect_state,
        }
    }

    pub fn without_openidconnect_session(
        &mut self,
    ) -> Session<CsrfToken, CookieValue<()>, TemporaryOpenIdConnectState> {
        if let Unchanged(None) = self.openidconnect_session {
            Session {
                csrf_token: self.csrf_token,
                openidconnect_session: CookieValue::Unchanged(Unchanged(())),
                temporary_openidconnect_state: self.temporary_openidconnect_state,
            }
        } else {
            Session {
                csrf_token: self.csrf_token,
                openidconnect_session: CookieValue::Changed(Changed(())),
                temporary_openidconnect_state: self.temporary_openidconnect_state,
            }
        }
    }
}

impl<
    CsrfToken: Cookiey + CookieyChanged,
    OpenIdConnectSession: Cookiey + CookieyChanged,
    TemporaryOpenIdConnectState: Cookiey + CookieyChanged,
> Session<CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState>
{
    pub fn with_temporary_openidconnect_state(
        &mut self,
        input: &OpenIdSession,
    ) -> Session<CsrfToken, OpenIdConnectSession, Changed<String>> {
        Session {
            csrf_token: self.csrf_token,
            openidconnect_session: self.openidconnect_session,
            temporary_openidconnect_state: Changed(serde_json::to_string(input).unwrap()),
        }
    }

    pub fn get_and_remove_temporary_openidconnect_state(
        self,
    ) -> Result<Session<CsrfToken, OpenIdConnectSession, Changed<()>>, AppError> {
        if let Unchanged(Some(temporary_openidconnect_state)) = self.temporary_openidconnect_state {
            Ok(Session {
                csrf_token: self.csrf_token,
                openidconnect_session: self.openidconnect_session,
                temporary_openidconnect_state: Changed(()),
            })
        } else {
            Err(AppError::OpenIdTokenNotFound)
        }
    }
}

// I think the csrf token needs to be signed/encrypted
impl<
    CsrfToken: Cookiey + CookieyChanged,
    OpenIdConnectSession: Cookiey + CookieyChanged,
    TemporaryOpenIdConnectState: Cookiey + CookieyChanged,
> Session<CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState>
{
    pub fn to_cookies() {
        Cookie::build(COOKIE_NAME_OPENIDCONNECT_SESSION)
            .build()
            .make_removal()
    }
}
