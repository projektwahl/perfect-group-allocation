pub mod client;

use std::future::Future;

use client::fetch_url;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
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
    let fut = run_server().await.unwrap();
    async move {
        fut.await.unwrap();
    }
}

#[tokio::main(flavor = "current_thread")]
#[allow(clippy::redundant_pub_crate)]
pub async fn bench_function(repeat: u64) {
    std::env::set_var(
        "DATABASE_URL",
        "postgres://postgres@localhost/pga?sslmode=disable",
    );
    let server_fut = test_server().await; // server doesn't terminate
    let client_fut = test_as_client(repeat);
    tokio::select! {
        val = server_fut => {
            println!("server completed first with {val:?}");
        }
        val = client_fut => {
            println!("client completed first with {val:?}");
        }
    };
}

#[library_benchmark]
#[bench::short(100)]
fn bench_client_server(value: u64) {
    bench_function(value);
}

library_benchmark_group!(
    name = bench_client_server_http;
    benchmarks = bench_client_server
);

main!(library_benchmark_groups = bench_client_server_http);
