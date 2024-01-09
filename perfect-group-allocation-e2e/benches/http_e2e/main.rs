use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use perfect_group_allocation_e2e::http_e2e::bench_client_server_function;

#[library_benchmark]
#[bench::short(100)]
fn bench_client_server(value: u64) {
    bench_client_server_function(value);
}

library_benchmark_group!(
    name = bench_client_server_http;
    benchmarks = bench_client_server
);

main!(library_benchmark_groups = bench_client_server_http);
