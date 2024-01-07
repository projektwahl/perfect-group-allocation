#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use webdriver_bidi::browsing_context::create::Type;
use webdriver_bidi::browsing_context::{self};
use webdriver_bidi::{Browser, SendCommand, WebDriver};

#[tokio::main]
pub async fn main() {
    if let Err(error) = inner_main().await {
        eprintln!("{error}");
    }
}

pub async fn inner_main() -> Result<(), webdriver_bidi::Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let driver = WebDriver::new(Browser::Chromium).await?;
    let mut session = driver.session_new().await?;
    let browsing_context = driver
        .send_command(
            browsing_context::create::Command {
                params: browsing_context::create::Parameters {
                    r#type: Type::Window,
                    reference_context: None,
                    background: Some(false),
                },
            },
            SendCommand::BrowsingContextCreate,
        )
        .await?;
    println!("{browsing_context:?}");
    let browsing_context = browsing_context.context.clone();
    let mut subscription = session.subscribe_logs(browsing_context.clone()).await?;
    let navigation = session
        .browsing_context_navigate(browsing_context, "https://www.google.com/".to_owned())
        .await?;
    println!("{navigation:?}");

    while let Ok(log) = subscription.recv().await {
        println!("received log message: {log:?}");
    }

    session.session_end().await?;

    Ok(())
}
