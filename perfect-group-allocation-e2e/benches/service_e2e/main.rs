use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use hyper::Request;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use perfect_group_allocation_backend::setup_server;
use tower_service::Service;

// podman run --rm --detach --name postgres-testing --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres

// TODO FIXME use black_box

#[tokio::main]
async fn hello_world(value: u64) {
    std::env::set_var(
        "DATABASE_URL",
        "postgres://postgres@localhost/pga?sslmode=disable",
    );
    let mut service = setup_server().await.unwrap();

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

#[library_benchmark]
#[bench::short(100)]
fn bench_client_server(value: u64) {
    hello_world(value);
}

library_benchmark_group!(
    name = bench_client_server_service;
    benchmarks = bench_client_server
);

main!(library_benchmark_groups = bench_client_server_service);
