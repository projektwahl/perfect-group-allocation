use std::pin::pin;

use async_zero_cost_templating::{html, TheStream};
use bytes::Bytes;
use futures_util::Stream;

pub fn main(page_title: Bytes) -> impl Stream<Item = Bytes> {
    // TODO FIXME implement self closing tags /> if they are valid in html5
    let html = html! {
    <!doctype html>
    <html lang="en">

    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <title>{page_title}</title>
        <link rel="icon" r#type="image/x-icon" href="/favicon.ico?v=1">
        <link rel="stylesheet" href="/bundle.css?v={{indexcss_version}}">
    </head>

    <body>
        <nav>
            <span>PGA</span>
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
                    <a href="/">Home</a>
                </li>
                <li>
                    <a href="/list">Projects</a>
                </li>
                <li>
                    if email {
                        <form method="post" action="/openidconnect-logout" enctype="application/x-www-form-urlencoded">
                            <input type="hidden" name="csrf_token" value="{{csrf_token}}">

                            <button class="submit-link" type="submit">Logout {email}</button>
                        </form>
                    } else {
                        <form method="post" action="/openidconnect-login" enctype="application/x-www-form-urlencoded">
                            <input type="hidden" name="csrf_token" value="{{csrf_token}}">

                            <button class="submit-link" type="submit">Login</button>
                        </form>
                    }
                </li>
            </ul>
        </nav>
        <main>
            { Bytes::from_static(b"") }
        </main>
    </body>

    </html>
    };
    TheStream::new(html)
}
