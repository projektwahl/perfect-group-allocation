pub mod client;

use std::hint::black_box;
use std::time::Duration;

use client::fetch_url;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use perfect_group_allocation_backend::error::AppError;
use perfect_group_allocation_backend::run_server;
use tokio::join;
use tokio::time::sleep;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type ReturnType = (
    std::result::Result<(), perfect_group_allocation_backend::error::AppError>,
    std::result::Result<
        (),
        std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>,
    >,
);
pub async fn test_server() -> std::result::Result<(), AppError> {
    run_server().await
}

pub async fn test_as_client(repeat: u64) -> Result<()> {
    sleep(Duration::from_millis(100)).await; // wait until server started hack
    for _ in 0..repeat {
        fetch_url("http://localhost:3000/".parse::<hyper::Uri>().unwrap()).await?;
    }
    Ok(())
}

#[tokio::main]
pub async fn bench_function(repeat: u64) -> ReturnType {
    let server_fut = test_server();
    let client_fut = test_as_client(repeat);
    join!(server_fut, client_fut)
}

#[library_benchmark]
#[bench::short(10)]
#[bench::long(30)]
fn bench_client_server(value: u64) -> ReturnType {
    black_box(bench_function(value))
}

library_benchmark_group!(
    name = bench_fibonacci_group;
    benchmarks = bench_client_server
);

main!(library_benchmark_groups = bench_fibonacci_group);
