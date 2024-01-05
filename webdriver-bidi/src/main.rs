#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use webdriver_bidi::webdriver::WebDriver;

#[tokio::main]
pub async fn main() -> Result<(), tokio_tungstenite::tungstenite::Error> {
    // firefox --profile /tmp/a --new-instance --remote-debugging-port 9222

    let driver = WebDriver::new().await?;
    let mut session = driver.session_new().await?;
    let browsing_context = session.browsing_context_get_tree().await?;
    println!("{browsing_context:?}");
    session.session_end().await?;

    Ok(())
}
