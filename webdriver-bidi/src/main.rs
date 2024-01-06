#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::time::Duration;

use tokio::time::sleep;
use webdriver_bidi::webdriver::WebDriver;

#[tokio::main]
pub async fn main() -> Result<(), webdriver_bidi::result::Error> {
    let driver = WebDriver::new().await?;
    println!("test");
    let mut session = driver.session_new().await?;
    println!("new session");
    let browsing_context = session.browsing_context_get_tree().await?;
    println!("{browsing_context:?}");
    let browsing_context = browsing_context.contexts[0].context.clone();
    session.session_subscribe(browsing_context.clone()).await?;
    let navigation = session
        .browsing_context_navigate(browsing_context, "https://www.google.com/".to_owned())
        .await?;
    println!("{navigation:?}");

    sleep(Duration::from_secs(5)).await;

    session.session_end().await?;

    Ok(())
}
