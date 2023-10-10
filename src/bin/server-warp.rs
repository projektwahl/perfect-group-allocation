#![deny(warnings)]

use warp::Filter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let html = include_str!("../../frontend/form.html");

    let routes = warp::path::end()
        .map(|| warp::reply::html(html.to_string()))
        .or(warp::fs::dir("./frontend"));

    println!("listening");

    warp::serve(routes)
        .tls()
        .cert_path(".lego/certificates/h3.selfmade4u.de.crt")
        .key_path(".lego/certificates/h3.selfmade4u.de.key")
        .run(([0, 0, 0, 0], 443))
        .await;
}
