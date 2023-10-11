// https://github.com/tokio-rs/axum/blob/v0.6.x/examples/low-level-rustls/src/main.rs
//! Run with
//!
//! ```not_rust
//! cargo run -p example-low-level-rustls
//! ```

use axum::{extract::ConnectInfo, response::IntoResponse, routing::get, Router};
use futures_util::future::poll_fn;
use http::header;
use hyper::server::{
    accept::Accept,
    conn::{AddrIncoming, Http},
};
use rustls_pemfile::{certs, ec_private_keys};
use std::{
    fs::File,
    io::BufReader,
    net::SocketAddr,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};
use tokio::net::TcpListener;
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};
use tower::MakeService;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    let rustls_config = rustls_server_config(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".lego/certificates/h3.selfmade4u.de.key"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".lego/certificates/h3.selfmade4u.de.crt"),
    );

    let acceptor = TlsAcceptor::from(rustls_config);

    let listener = TcpListener::bind("[::]:443").await.unwrap();
    let mut listener = AddrIncoming::from_listener(listener).unwrap();

    println!("listening on {}", listener.local_addr().to_string());

    let protocol = Arc::new(Http::new());

    let mut app = Router::new()
        .route("/", get(handler))
        .into_make_service_with_connect_info::<SocketAddr>();

    loop {
        let stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx))
            .await
            .unwrap()
            .unwrap();

        let acceptor = acceptor.clone();

        let protocol = protocol.clone();

        let svc = app.make_service(&stream);

        tokio::spawn(async move {
            if let Ok(stream) = acceptor.accept(stream).await {
                let _ = protocol.serve_connection(stream, svc.await.unwrap()).await;
            }
        });
    }
}

async fn handler(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    (
        [(header::ALT_SVC, r#"h3=":443"; ma=86400"#)],
        addr.to_string(),
    )
}

fn rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Arc<ServerConfig> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let key = PrivateKey(ec_private_keys(&mut key_reader).unwrap().remove(0));
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
