use std::marker::PhantomData;

use cookie::{Cookie, CookieJar, SameSite};
use http::header::COOKIE;
use http::Request;
use perfect_group_allocation_openidconnect::OpenIdSession;
use rand::{thread_rng, Rng as _};

use crate::error::AppError;

const COOKIE_NAME_CSRF_TOKEN: &str = "__Host_csrf_token";
const COOKIE_NAME_OPENIDCONNECT_SESSION: &str = "__Host_openidconnect_session";
const COOKIE_NAME_TEMPORARY_OPENIDCONNECT_STATE: &str = "__Host_temporary_openidconnect_state";

pub struct Changed<T>(T);

pub struct Unchanged<T>(T);

pub enum CookieValue<T> {
    Unchanged(Unchanged<T>),
    Changed(Changed<T>),
}

// we don't want to store cookies we don't need
#[must_use]
pub struct Session<CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState> {
    pub csrf_token: CsrfToken,
    pub openidconnect_session: OpenIdConnectSession,
    pub temporary_openidconnect_state: TemporaryOpenIdConnectState,
}

impl
    Session<Unchanged<Option<String>>, Unchanged<Option<String>>, Unchanged<Option<OpenIdSession>>>
{
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

impl<OpenIdConnectSession, TemporaryOpenIdConnectState>
    Session<Unchanged<Option<String>>, OpenIdConnectSession, TemporaryOpenIdConnectState>
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

impl<CsrfToken, TemporaryOpenIdConnectState>
    Session<CsrfToken, Unchanged<Option<String>>, TemporaryOpenIdConnectState>
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

impl<CsrfToken, OpenIdConnectSession> Session<CsrfToken, OpenIdConnectSession, Option<String>> {
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
        &mut self,
    ) -> Result<OpenIdSession, AppError> {
        let return_value = match self
            .private_cookies
            .get(Self::COOKIE_NAME_OPENIDCONNECT)
            .map(|cookie| serde_json::from_str(cookie.value()))
            .ok_or(AppError::OpenIdTokenNotFound)
        {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => return Err(error.into()),
            Err(error) => return Err(error),
        };
        let cookie = Cookie::build((Self::COOKIE_NAME_OPENIDCONNECT, ""))
            .http_only(true)
            .same_site(SameSite::Lax) // needed because top level callback is cross-site
            /*.secure(true) */;
        self.private_cookies.remove(cookie);
        Ok(return_value)
    }
}

// I think the csrf token needs to be signed/encrypted
impl<CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState>
    Session<CsrfToken, OpenIdConnectSession, TemporaryOpenIdConnectState>
{
    pub fn to_cookies() {
        Cookie::build(COOKIE_NAME_OPENIDCONNECT_SESSION)
            .build()
            .make_removal()
    }
}
