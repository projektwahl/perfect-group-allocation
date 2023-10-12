mod entities;
mod migrator;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::migrator::Migrator;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use axum::ServiceExt;
use hyper::server::conn::AddrIncoming;
use hyper::server::conn::Http;
use rustls_pemfile::certs;
use rustls_pemfile::pkcs8_private_keys;
use sea_orm::ConnectionTrait;
use sea_orm::Database;
use sea_orm::DbBackend;
use sea_orm::DbErr;
use sea_orm::Statement;
use sea_orm_migration::MigratorTrait;
use sea_orm_migration::SchemaManager;
use tokio::net::TcpListener;
use tokio_rustls::rustls::Certificate;
use tokio_rustls::rustls::PrivateKey;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DATABASE_URL: &str = "sqlite:./sqlite.db?mode=rwc";
const DB_NAME: &str = "pga";

async fn handler() -> &'static str {
    "Hello, World!"
}

fn rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Arc<ServerConfig> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let key = PrivateKey(pkcs8_private_keys(&mut key_reader).unwrap().remove(0));
    let certs = certs(&mut cert_reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("bad certificate/key");

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Arc::new(config)
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_tls_rustls=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
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

    let rustls_config = rustls_server_config(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("self_signed_certs")
            .join("key.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("self_signed_certs")
            .join("cert.pem"),
    );

    let html = include_str!("../../../frontend/form.html");

    //  .cert_path(".lego/certificates/h3.selfmade4u.de.crt")
    //  .key_path(".lego/certificates/h3.selfmade4u.de.key")

    let acceptor = TlsAcceptor::from(rustls_config);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let mut listener = AddrIncoming::from_listener(listener).unwrap();

    let protocol = Arc::new(Http::new());

    let mut app = Router::<()>::new()
        .route("/", get(handler))
        .into_make_service();

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
