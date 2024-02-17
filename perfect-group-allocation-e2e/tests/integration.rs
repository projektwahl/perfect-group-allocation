use std::process::ExitStatus;

// cargo test -p perfect-group-allocation-e2e --test integration
// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
#[tokio::test]
async fn it_adds_two() {
    println!("gonna run {}", env!("CARGO_BIN_EXE_webdriver"));

    let status = tokio::process::Command::new("./runner.sh")
        .current_dir("..")
        .arg(env!("CARGO_BIN_EXE_webdriver"))
        .status()
        .await
        .unwrap();

    assert!(status.success(), "process failed");
}
