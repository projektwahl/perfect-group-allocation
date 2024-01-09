use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use hyper::Request;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use perfect_group_allocation_backend::setup_server;
use tower_service::Service;

// podman run --rm --detach --name postgres-testing --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres

// TODO FIXME use black_box

#[tokio::main(flavor = "current_thread")]
async fn bench_client_server_function(value: u64) {
    let mut service = setup_server("postgres://postgres@localhost/pga?sslmode=disable")
        .await
        .unwrap();
    for _ in 0..value {
        let mut service = service
            .call(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(0, 0, 0, 0),
                3000,
            )))
            .await
            .unwrap();

        // TODO FIXME check response
        service
            .call(
                Request::builder()
                    .uri("http://localhost:3000/")
                    .body(http_body_util::Empty::new())
                    .unwrap(),
            )
            .await
            .unwrap();
    }
}
