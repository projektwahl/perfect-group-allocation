// syscalls are bad for determinism (also println in dependencies)
// use strace to find used syscalls
// hashmaps are bad for determinism
// https://github.com/rust-lang/cargo/issues/3670
// https://github.com/rust-lang/cargo/issues/1924
