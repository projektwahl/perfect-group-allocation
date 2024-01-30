use tracing::{info, trace, Level};
use tracing_subscriber::FmtSubscriber;
use webdriver_bidi::browsing_context::create::Type;
use webdriver_bidi::browsing_context::{self};
use webdriver_bidi::protocol::EmptyParams;
use webdriver_bidi::session::CapabilitiesRequest;
use webdriver_bidi::{session, Browser, SendCommand, WebDriver};

#[tokio::main]
pub async fn main() {
    if let Err(error) = inner_main().await {
        trace!("{error}");
    }
}

pub async fn inner_main() -> Result<(), webdriver_bidi::Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let driver = WebDriver::new(Browser::Chromium).await?;
    let _session = driver
        .send_command(
            SendCommand::SessionNew,
            session::new::Command {
                params: session::new::Parameters {
                    capabilities: CapabilitiesRequest {
                        always_match: None,
                        first_match: None,
                    },
                },
            },
        )
        .await?;
    let browsing_context = driver
        .send_command(
            SendCommand::BrowsingContextCreate,
            browsing_context::create::Command {
                params: browsing_context::create::Parameters {
                    r#type: Type::Window,
                    reference_context: None,
                    background: Some(false),
                },
            },
        )
        .await?;
    let browsing_context = browsing_context.context.clone();
    let mut subscription = driver
        .request_subscribe(
            SendCommand::SubscribeGlobalLogs,
            Some(browsing_context.clone()),
        )
        .await?;
    let _navigation = driver
        .send_command(
            SendCommand::BrowsingContextNavigate,
            browsing_context::navigate::Command {
                params: browsing_context::navigate::Parameters {
                    context: browsing_context,
                    url: "https://www.google.com/".to_owned(),
                    wait: Some(browsing_context::ReadinessState::Complete),
                },
            },
        )
        .await?;

    while let Ok(log) = subscription.recv().await {
        info!("received log message: {log:?}");
    }

    driver
        .send_command(
            SendCommand::SessionEnd,
            session::end::Command {
                params: EmptyParams::default(),
            },
        )
        .await?;

    Ok(())
}
