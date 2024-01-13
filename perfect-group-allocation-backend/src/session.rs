use cookie::{Cookie, CookieJar, SameSite};
use perfect_group_allocation_openidconnect::OpenIdSession;

use crate::error::AppError;

#[derive(Clone)]
#[must_use]
pub struct Session {
    private_cookies: CookieJar, // TODO FIXME
}

// I think the csrf token needs to be signed/encrypted
impl Session {
    //const COOKIE_NAME_OPENIDCONNECT: &'static str = "__Host-openidconnect";
    //const COOKIE_NAME_SESSION: &'static str = "__Host-session";
    const COOKIE_NAME_OPENIDCONNECT: &'static str = "openidconnect";
    const COOKIE_NAME_SESSION: &'static str = "session";

    pub fn new(private_cookies: CookieJar) -> Self {
        let mut session = Self { private_cookies };
        if session.optional_session().is_none() {
            session.set_openid_session(None);
        }
        session
    }

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
