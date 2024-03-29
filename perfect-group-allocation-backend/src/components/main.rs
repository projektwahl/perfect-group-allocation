use std::borrow::Cow;

use async_zero_cost_templating::html;
use futures_util::Future;
use perfect_group_allocation_config::Config;
use perfect_group_allocation_openidconnect::id_token_claims;

use crate::session::{IntoCookieValue, Session};

#[allow(clippy::future_not_send)]
pub async fn main<
    'a,
    F: Future<Output = ()> + Send + 'a,
    OpenIdConnectSession: IntoCookieValue + Clone,
    TemporaryOpenIdConnectState: IntoCookieValue,
>(
    tx: tokio::sync::mpsc::Sender<Cow<'a, str>>,
    page_title: Cow<'a, str>,
    session: &'a Session<OpenIdConnectSession, TemporaryOpenIdConnectState>,
    config: &'a Config,
    inner: F,
) {
    // TODO support if let and while let and while and normal for?

    let openidconnect_session = session.openidconnect_session().into_cookie_value();
    let csrf_token = session.csrf_token();
    let email;
    if let Some(openidconnect_session) = openidconnect_session {
        let claims = id_token_claims(config, openidconnect_session).await; // TODO FIXME avoid crashing at all costs because this is used on the error page
        email = claims
            .ok()
            .and_then(|value| value.email().map(|email| email.to_string()));
    } else {
        email = None;
    }
    let indexcss_version = Cow::Borrowed("42");

    html! {
    <!doctype html>
    <html lang="en">

    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <title>(page_title)</title>
        <link rel="icon" type="image/x-icon" href="/favicon.ico?v=1">
        <link rel="stylesheet" href=["/bundle.css?v=" (indexcss_version)]>
    </head>

    <body>
        <nav>
            <span>"PGA"</span>
            <input id="open-nav" type="checkbox" checked>
            <label for="open-nav" class="hamb">
                <svg viewBox="0 0 100 100">
                    <rect y="10" width="100" height="20"></rect>
                    <rect y="40" width="100" height="20"></rect>
                    <rect y="70" width="100" height="20"></rect>
                </svg>
            </label>
            <ul>
                <li>
                    <a href="/">"Home"</a>
                </li>
                <li>
                    <a href="/list">"Projects"</a>
                </li>
                <li>
                    <form method="post" action="/" enctype="application/x-www-form-urlencoded">
                        <input type="hidden" name="csrf_token" value=[(Cow::Owned(csrf_token.clone()))]>

                        <input name="title" type="hidden" value=[(Cow::Borrowed(""))]>
                        <input name="description" type="hidden" value=[(Cow::Borrowed(""))] >

                        <button class="submit-link" type="submit">"Create project"</button>
                    </form>
                </li>
                <li>
                    if email.is_some() {
                        <form method="post" action="/openidconnect-logout" enctype="application/x-www-form-urlencoded">
                            <input type="hidden" name="csrf_token" value=[(Cow::Owned(csrf_token.clone()))]>

                            <button id="logout-button" class="submit-link" type="submit">"Logout "(email.unwrap().into())</button>
                        </form>
                    } else {
                        <form method="post" action="/openidconnect-login" enctype="application/x-www-form-urlencoded">
                            <input type="hidden" name="csrf_token" value=[(Cow::Owned(csrf_token.clone()))]>

                            <button id="login-button" class="submit-link" type="submit">"Login"</button>
                        </form>
                    }
                </li>
            </ul>
        </nav>
        <main>
            { inner.await; }
        </main>
    </body>

    </html>
    }
}
