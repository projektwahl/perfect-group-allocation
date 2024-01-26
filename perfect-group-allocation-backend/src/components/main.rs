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
        <main>
            { Bytes::from_static(b"") }
        </main>
    </body>

    </html>
    };
    TheStream::new(html)
}
