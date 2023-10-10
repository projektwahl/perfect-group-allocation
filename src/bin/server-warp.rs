#![deny(warnings)]

use bytes::BufMut;
use futures_util::TryStreamExt;
use warp::{
    filters::{compression::brotli, multipart::FormData},
    Filter,
};

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

    // we could rely on the order?
    let route2 = warp::path::end()
        .and(warp::post())
        .and(warp::filters::multipart::form())
        .and_then(|form: FormData| async move {
            let field_names: Vec<_> = form
                .and_then(|mut field| async move {
                    let mut bytes: Vec<u8> = Vec::new();

                    // field.data() only returns a piece of the content, you should call over it until it replies None
                    while let Some(content) = field.data().await {
                        let content = content.unwrap();
                        bytes.put(content);
                    }
                    Ok((
                        field.name().to_string(),
                        String::from_utf8_lossy(&*bytes).to_string(),
                    ))
                })
                .try_collect()
                .await
                .unwrap();

            Ok::<_, warp::Rejection>(format!("{:?}", field_names))
        });

    let route3 = warp::fs::dir("./frontend");

    println!("listening");

    warp::serve(route1.or(route2).or(route3).with(brotli()))
        .tls()
        .cert_path(".lego/certificates/h3.selfmade4u.de.crt")
        .key_path(".lego/certificates/h3.selfmade4u.de.key")
        .run(([0, 0, 0, 0], 443))
        .await;
}
