#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::time::Duration;

use futures::future::FutureExt as _;
use tokio::time::sleep;
use webdriver_bidi::webdriver::WebDriver;

#[tokio::main]
pub async fn main() -> Result<(), tokio_tungstenite::tungstenite::Error> {
    let driver = WebDriver::new().await?;
    let mut session = driver.session_new().await?;
    let browsing_context = session.browsing_context_get_tree().await?;
    println!("{browsing_context:?}");
    let browsing_context = browsing_context.contexts[0].context.clone();
    session.session_subscribe(browsing_context.clone()).await?;
    let _navigation = session
        .browsing_context_navigate(browsing_context, "https://www.google.com/".to_owned())
        .await?;

    sleep(Duration::from_secs(5)).await;

    session.session_end().await?;

    Ok(())
}
