// https://github.com/hyperium/hyper/blob/master/examples/client.rs

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::rt::TokioIo;

use tokio::net::TcpStream;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn fetch_url(url: hyper::Uri) -> Result<()> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{host}:{port}");
    let stream = TcpStream::connect(addr).await?;
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


use perfect_group_allocation_backend::run_server;

// podman run --rm --detach --name postgres-testing --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres

// TODO FIXME use black_box

pub async fn test_as_client(repeat: u64) {
    for _ in 0..repeat {
        fetch_url("http://localhost:3000/".parse::<hyper::Uri>().unwrap())
            .await
            .unwrap();
    }
}

pub async fn test_server() -> impl Future<Output = ()> {
    let fut = run_server("postgres://postgres@localhost/pga?sslmode=disable".to_owned())
        .await
        .unwrap();
    async move {
        fut.await.unwrap();
    }
}

#[tokio::main(flavor = "current_thread")]
#[allow(clippy::redundant_pub_crate)]
pub async fn bench_client_server_function_http(repeat: u64) {
    let server_fut = test_server().await; // server doesn't terminate
    let client_fut = test_as_client(repeat);
    tokio::select! {
        _val = server_fut => {
        }
        _val = client_fut => {
        }
    };
}
