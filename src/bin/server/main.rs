mod entities;
mod migrator;
use entities::{prelude::*, *};

use bytes::BufMut;
use futures_util::StreamExt;
use futures_util::TryFutureExt;
use futures_util::TryStreamExt;
use sea_orm::ConnectionTrait;
use sea_orm::Database;
use sea_orm::DbBackend;
use sea_orm::DbErr;
use sea_orm::Statement;
use sea_orm_migration::MigratorTrait;
use sea_orm_migration::SchemaManager;
use warp::{
    filters::{compression::brotli, multipart::FormData},
    Filter,
};

use crate::migrator::Migrator;

const DATABASE_URL: &str = "sqlite:./sqlite.db?mode=rwc";
const DB_NAME: &str = "pga";

#[derive(Debug)]
struct InvalidParameter;

impl warp::reject::Reject for InvalidParameter {}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    let db = Database::connect(DATABASE_URL).await?;

    let db = &match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", DB_NAME),
            ))
            .await?;

            let url = format!("{}/{}", DATABASE_URL, DB_NAME);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            let err_already_exists = db
                .execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE \"{}\";", DB_NAME),
                ))
                .await;

            if let Err(err) = err_already_exists {
                println!("{err:?}");
            }

            let url = format!("{}/{}", DATABASE_URL, DB_NAME);
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    // sea-orm-cli generate entity -u sqlite:./sqlite.db?mode=rwc -o src/bin/server/entities

    let schema_manager = SchemaManager::new(db); // To investigate the schema

    Migrator::up(db, None).await?;
    assert!(schema_manager.has_table("project_history").await?);

    // handlebars?
    let html = include_str!("../../../frontend/form.html");

    let route1 = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(html.to_string()));

    // we could rely on the order?
    let route2 = warp::path::end()
        .and(warp::post())
        .and(warp::filters::multipart::form())
        .then(|mut form: FormData| async move {
            while let Some(item) = form.try_next().await.map_err(|err| )? {}
            /*let field_names: Vec<_> = form
                .and_then(|field| {
                    let name = field.name().to_string();

                    match name.as_ref() {
                        "title" => field.stream().try_fold(Vec::new(), |mut vec, data| {
                            vec.put(data);
                            async move { Ok(vec) }
                        }),
                        "description" => field.stream().try_fold(Vec::new(), |mut vec, data| {
                            vec.put(data);
                            async move { Ok(vec) }
                        }),
                        _ => {
                            panic!();
                        }
                    }

                    value.map_ok(move |vec| (name, vec))
                })
                .try_collect()
                .await
                .unwrap();

            let project = project_history::ActiveModel {
                ..Default::default()
            };

            Ok::<_, warp::Rejection>(format!("{:?}", field_names))*/
            Ok::<_, warp::Error>("")
        });

    let route3 = warp::fs::dir("./frontend");

    println!("listening");

    warp::serve(route1.or(route2).or(route3).with(brotli()))
        .tls()
        .cert_path(".lego/certificates/h3.selfmade4u.de.crt")
        .key_path(".lego/certificates/h3.selfmade4u.de.key")
        .run(([0, 0, 0, 0], 8443)) // for http3 a port < 1024 is needed
        .await;

    Ok(())
}
