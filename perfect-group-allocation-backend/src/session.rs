use std::marker::PhantomData;

use cookie::{Cookie, CookieJar, SameSite};
use http::header::COOKIE;
use http::Request;
use perfect_group_allocation_openidconnect::OpenIdSession;

use crate::error::AppError;

const COOKIE_NAME_OPENIDCONNECT: &str = "__Host-openidconnect";
const COOKIE_NAME_SESSION: &str = "__Host-session";

pub enum CookieValue<T> {
    Unchanged(T),
    Changed(T),
}

// we don't want to store cookies we don't need
#[must_use]
pub struct Session<S> {
    session_id: CookieValue<Option<String>>,
    temporary_openid_state: CookieValue<Option<OpenIdSession>>,
    openid_session: CookieValue<Option<String>>,
}

impl Session<Option<String>> {
    pub fn new<T>(request: Request<T>) -> Self {
        let mut new = Self {
            session_id: CookieValue::Unchanged(None),
            openid_session: CookieValue::Unchanged(None),
            temporary_openid_state: CookieValue::Unchanged(None),
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
                COOKIE_NAME_SESSION => {
                    new.session_id = CookieValue::Unchanged(Some(cookie.value().to_owned()))
                }
                COOKIE_NAME_OPENIDCONNECT => {
                    new.openid_session = CookieValue::Unchanged(Some(cookie.value().to_owned()))
                }
                _ => {}
            });
        new
    }
}

// I think the csrf token needs to be signed/encrypted
impl<S> Session<S> {
    fn optional_session(&self) -> Option<(String, Option<String>)> {
        self.private_cookies
            .get(Self::COOKIE_NAME_SESSION)
            .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
    }

    /// first return value is `session_id`, second is openid session
    #[must_use]
    pub fn session(&self) -> (String, Option<String>) {
        #[expect(clippy::unwrap_used, reason = "set in constructor so has to exist")]
        self.optional_session().unwrap()
    }

    pub fn set_openid_session(&mut self, input: Option<String>) -> (String, Option<String>) {
        /*
        let session_id: String = thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();*/
        let session_id = "1337".to_string();

        let value = (session_id.clone(), &input);
        let cookie = Cookie::build((Self::COOKIE_NAME_SESSION, serde_json::to_string(&value).unwrap()))
            .http_only(true)
            .same_site(SameSite::Lax) // openid-redirect is a cross-site-redirect
            /*.secure(true) */;
        self.private_cookies.add(cookie);
        (session_id, input)
    }

    pub fn set_openidconnect(&mut self, input: &OpenIdSession) -> Result<(), AppError> {
        let cookie = Cookie::build((
            Self::COOKIE_NAME_OPENIDCONNECT,
            serde_json::to_string(input)?,
        ))
        .http_only(true)
        .same_site(SameSite::Lax) // needed because top level callback is cross-site
            /*.secure(true) */;
        self.private_cookies.add(cookie);
        Ok(())
    }

    pub fn get_and_remove_openidconnect(&mut self) -> Result<OpenIdSession, AppError> {
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
