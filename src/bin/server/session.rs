use std::convert::Infallible;

use axum::response::IntoResponseParts;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::PrivateCookieJar;
use oauth2::PkceCodeVerifier;
use openidconnect::Nonce;
use rand::{thread_rng, Rng};

#[derive(Clone)]
pub struct Session {
    private_cookies: PrivateCookieJar,
}

impl Session {
    const COOKIE_NAME_OPENID_NONCE: &'static str = "__Host-openid-nonce";
    const COOKIE_NAME_PKCE_VERIFIER: &'static str = "__Host-openid-pkce-verifier";

    pub fn new(private_cookies: PrivateCookieJar) -> Self {
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
        let cookie = Cookie::build(Self::COOKIE_NAME_PKCE_VERIFIER, verifier.secret())
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Strict)
            .secure(true)
            .finish();
        self.private_cookies = self.private_cookies.clone().add(cookie);
    }

    pub fn openid_pkce_verifier(&self) -> PkceCodeVerifier {
        self.private_cookies
            .get(Self::COOKIE_NAME_PKCE_VERIFIER)
            .map(|c| PkceCodeVerifier::new(c.value().to_string()))
            .unwrap()
    }

    pub fn set_openid_nonce(&mut self, nonce: &Nonce) {
        let cookie = Cookie::build(Self::COOKIE_NAME_OPENID_NONCE, nonce.secret())
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Strict)
            .secure(true)
            .finish();
        self.private_cookies = self.private_cookies.clone().add(cookie);
    }

    pub fn openid_nonce(&self) -> Nonce {
        self.private_cookies
            .get(Self::COOKIE_NAME_OPENID_NONCE)
            .map(|c| Nonce::new(c.value().to_string()))
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
