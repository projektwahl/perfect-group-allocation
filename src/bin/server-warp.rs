#![deny(warnings)]

use warp::{filters::compression::brotli, Filter};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    let html = include_str!("../../frontend/form.html");

    let route1 = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(html.to_string()));

    let route2 = warp::fs::dir("./frontend");

    println!("listening");

    warp::serve(route1.or(route2).with(brotli()))
        .tls()
        .cert_path(".lego/certificates/h3.selfmade4u.de.crt")
        .key_path(".lego/certificates/h3.selfmade4u.de.key")
        .run(([0, 0, 0, 0], 443))
        .await;
}
