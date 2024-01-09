pub mod http_e2e;
pub mod service_e2e;

// syscalls are bad for determinism (also println in dependencies)
// use strace to find used syscalls
// hashmaps are bad for determinism
