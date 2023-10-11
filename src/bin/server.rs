#![deny(warnings)]

use bytes::BufMut;
use futures_util::TryFutureExt;
use futures_util::TryStreamExt;
use sea_orm::Database;
use sea_orm::DbErr;
use warp::{
    filters::{compression::brotli, multipart::FormData},
    Filter,
};

const DATABASE_URL: &str = "sqlite:./sqlite.db?mode=rwc";
const DB_NAME: &str = "bakeries_db";

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    let db = Database::connect(DATABASE_URL).await?;

    let html = include_str!("../../frontend/form.html");

    let route1 = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(html.to_string()));

    // we could rely on the order?
    let route2 = warp::path::end()
        .and(warp::post())
        .and(warp::filters::multipart::form())
        .and_then(|form: FormData| async {
            let field_names: Vec<_> = form
                .and_then(|field| {
                    let name = field.name().to_string();

                    let value = field.stream().try_fold(Vec::new(), |mut vec, data| {
                        vec.put(data);
                        async move { Ok(vec) }
                    });
                    value.map_ok(move |vec| (name, vec))
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

    Ok(())
}
