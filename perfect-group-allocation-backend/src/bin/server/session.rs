use core::convert::Infallible;

use axum::extract::FromRequestParts;
use axum::response::IntoResponseParts;
use axum::{async_trait, RequestPartsExt};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::PrivateCookieJar;
use chrono::{DateTime, Utc};
use oauth2::{PkceCodeVerifier, RefreshToken};
use openidconnect::{EndUserEmail, Nonce};
use rand::{thread_rng, Rng};

use crate::error::AppError;

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
#[must_use]
pub struct Session {
    private_cookies: PrivateCookieJar,
}

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
    axum_extra::extract::cookie::Key: axum::extract::FromRef<S>,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::new(
            parts
                .extract_with_state::<PrivateCookieJar, S>(state)
                .await?,
        ))
    }
}

impl Session {
    //const COOKIE_NAME_OPENIDCONNECT: &'static str = "__Host-openidconnect";
    //const COOKIE_NAME_SESSION: &'static str = "__Host-session";
    const COOKIE_NAME_OPENIDCONNECT: &'static str = "openidconnect";
    const COOKIE_NAME_SESSION: &'static str = "session";

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
        #[expect(clippy::unwrap_used, reason = "set in constructor so has to exist")]
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
        let cookie = Cookie::build((Self::COOKIE_NAME_SESSION, test_to_string(&value)))
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax) // openid-redirect is a cross-site-redirect
            /*.secure(true) */;
        self.private_cookies = self.private_cookies.clone().add(cookie);
        (session_id, input)
    }

    pub fn set_openidconnect(
        &mut self,
        input: &(&PkceCodeVerifier, &Nonce, &oauth2::CsrfToken),
    ) -> Result<(), AppError> {
        let cookie = Cookie::build((
            Self::COOKIE_NAME_OPENIDCONNECT,
            serde_json::to_string(input)?,
        ))
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax) // needed because top level callback is cross-site
            /*.secure(true) */;
        self.private_cookies = self.private_cookies.clone().add(cookie);
        Ok(())
    }

    pub fn get_and_remove_openidconnect(
        &mut self,
    ) -> Result<(PkceCodeVerifier, Nonce, oauth2::CsrfToken), AppError> {
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
            .same_site(axum_extra::extract::cookie::SameSite::Lax) // needed because top level callback is cross-site
            /*.secure(true) */;
        self.private_cookies = self.private_cookies.clone().remove(cookie);
        Ok(return_value)
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
