// https://github.com/hyperium/hyper/blob/master/examples/client.rs
// https://github.com/rustls/tokio-rustls/blob/main/examples/client.rs
// maybe use https://docs.rs/hyper-rustls/0.26.0/hyper_rustls/, we may need low level control though to test stuff like sending headers and stopping sending then etc.

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::rt::TokioIo;
use perfect_group_allocation_backend::error::AppError;
use perfect_group_allocation_config::{get_config, Config, OpenIdConnectConfig, TlsConfig};
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::CertificateDer;
use tokio_rustls::rustls::{pki_types, RootCertStore};
use tokio_rustls::{rustls, TlsConnector};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn fetch_url(url: hyper::Uri) -> Result<()> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(443);
    let addr = format!("{host}:{port}");

    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(todo!())
        .map(|value| match value {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Box::new(AppError::TlsCertificate(err)) as Box<dyn std::error::Error>),
        })
        .collect::<Result<Vec<CertificateDer<'static>>>>()?;

    let mut root_store = RootCertStore::empty();
    root_store.add_parsable_certificates(certs);

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));

    let stream = TcpStream::connect(&addr).await?;

    let domain = pki_types::ServerName::try_from(host)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid dnsname"))?
        .to_owned();

    let stream = connector.connect(domain, stream).await?;

    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move { if let Err(_err) = conn.await {} });

    let authority = url.authority().unwrap().clone();

    let request = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let mut response = sender.send_request(request).await?;

    // Stream the body, writing each chunk to stdout as we get it
    // (instead of buffering and printing at the end).
    while let Some(next) = response.frame().await {
        let frame = next?;
        if let Some(_chunk) = frame.data_ref() {}
    }

    Ok(())
}

use std::future::Future;
use std::sync::Arc;

use perfect_group_allocation_backend::setup_http2_http3_server;

// podman run --rm --detach --name postgres-testing --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres

// TODO FIXME use black_box

pub async fn test_as_client(repeat: u64) {
    for _ in 0..repeat {
        fetch_url(
            "https://perfect-group-allocation"
                .parse::<hyper::Uri>()
                .unwrap(),
        )
        .await
        .unwrap();
    }
}

#[tokio::main(flavor = "current_thread")]
#[allow(clippy::redundant_pub_crate)]
pub async fn bench_client_server_function_http(repeat: u64) {
    test_as_client(repeat).await;
}
