use core::slice;

use iai_callgrind::{library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig};
use libc::{c_uint, size_t, ssize_t};
use perfect_group_allocation_e2e::service_e2e::bench_client_server_function_service;
use rand::{RngCore, SeedableRng};

/// # Safety
/// Totally unsafe.
#[allow(unsafe_code)]
#[no_mangle]
pub unsafe extern "C" fn getrandom(buf: *mut u8, buflen: size_t, _flags: c_uint) -> ssize_t {
    let slice = unsafe { slice::from_raw_parts_mut(buf, buflen) };
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    rng.fill_bytes(slice);
    buflen.try_into().unwrap()
}

#[library_benchmark]
#[bench::short(1)]
fn bench_client_server(value: u64) {
    bench_client_server_function_service(value);
}

library_benchmark_group!(
    name = bench_client_server_service;
    benchmarks = bench_client_server
);

main!(
    config = LibraryBenchmarkConfig::default().pass_through_env("RUST_BACKTRACE");
    library_benchmark_groups = bench_client_server_service
);
