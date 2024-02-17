// cargo test -p perfect-group-allocation-e2e --test integration
// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
#[tokio::test]
async fn it_adds_two() {
    panic!("gonna run {}", env!("CARGO_BIN_EXE_webdriver"));

    Command::new("./runner.sh")
        .args(env!("CARGO_BIN_EXE_webdriver"))
        .status()
        .await
        .unwrap();
}
