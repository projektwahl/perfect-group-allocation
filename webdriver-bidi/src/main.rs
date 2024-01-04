#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
use webdriver_bidi::WebDriverBiDi;

#[tokio::main]
pub async fn main() -> Result<(), tokio_tungstenite::tungstenite::Error> {
    let driver = WebDriverBiDi::new().await?;
    let session = driver.create_session().await?;
    println!("{session:?}");

    Ok(())
}
