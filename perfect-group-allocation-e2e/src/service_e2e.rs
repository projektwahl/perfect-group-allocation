use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::service::Service as _;
use hyper::Request;
use perfect_group_allocation_backend::setup_server;

// podman run --rm --detach --name postgres-testing --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres

// TODO FIXME use black_box

#[tokio::main(flavor = "current_thread")]
pub async fn bench_client_server_function_service(value: u64) {
    let mut service = setup_server("postgres://postgres@localhost/pga?sslmode=disable")
        .await
        .unwrap();
    for _ in 0..value {
        // TODO FIXME check response
        service
            .call(
                Request::builder()
                    .uri("http://localhost:3000/")
                    .body(http_body_util::Empty::new().map_err(|error| match error {}))
                    .unwrap(),
            )
            .await
            .unwrap();
    }
}
