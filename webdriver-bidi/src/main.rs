#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use webdriver_bidi::webdriver::WebDriver;

#[tokio::main]
pub async fn main() -> Result<(), tokio_tungstenite::tungstenite::Error> {
    let driver = WebDriver::new().await?;
    let session = driver.session_new().await?;
    println!("{session:?}");

    session.session_end().await?;
    println!("session ended");

    Ok(())
}
