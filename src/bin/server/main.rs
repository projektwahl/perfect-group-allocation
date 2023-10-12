mod entities;
mod migrator;
use std::net::SocketAddr;

use axum::routing::get;
use axum::routing::post;
use axum::Router;
use sea_orm::ConnectionTrait;
use sea_orm::Database;
use sea_orm::DbBackend;
use sea_orm::DbErr;
use sea_orm::Statement;
use sea_orm_migration::MigratorTrait;
use sea_orm_migration::SchemaManager;

use crate::migrator::Migrator;

const DATABASE_URL: &str = "sqlite:./sqlite.db?mode=rwc";
const DB_NAME: &str = "pga";

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

    let html = include_str!("../../../frontend/form.html");

    //  .cert_path(".lego/certificates/h3.selfmade4u.de.crt")
    //  .key_path(".lego/certificates/h3.selfmade4u.de.key")

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
