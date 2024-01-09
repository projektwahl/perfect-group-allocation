use perfect_group_allocation_backend::run_server;
use tracing::{info, Level};
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use webdriver_bidi::browsing_context::{self, CssLocator};
use webdriver_bidi::{session, Browser, SendCommand, WebDriver};

#[tokio::main]
pub async fn main() -> Result<(), webdriver_bidi::Error> {
    let fmt_layer = tracing_subscriber::fmt::layer();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug,webdriver_bidi=trace"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let fut = run_server("postgres://postgres@localhost/pga?sslmode=disable".to_owned())
        .await
        .unwrap();
    tokio::spawn(async move {
        fut.await.unwrap();
    });

    let driver = WebDriver::new(Browser::Firefox).await?;
    let _session = driver
        .send_command(
            SendCommand::SessionNew,
            session::new::Command {
                params: session::new::Parameters {
                    capabilities: session::CapabilitiesRequest {
                        always_match: None,
                        first_match: None,
                    },
                },
            },
        )
        .await?;

    let browsing_context = driver
        .send_command(
            SendCommand::BrowsingContextGetTree,
            browsing_context::get_tree::Command {
                params: browsing_context::get_tree::Parameters {
                    max_depth: None,
                    root: None,
                },
            },
        )
        .await?;
    info!("{:?}", browsing_context);
    let browsing_context = browsing_context.contexts.0[0].context.clone();
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
                    context: browsing_context.clone(),
                    url: "http://localhost:3000".to_owned(),
                    wait: Some(browsing_context::ReadinessState::Complete),
                },
            },
        )
        .await?;

    let nodes = driver
        .send_command(
            SendCommand::BrowsingContextLocateNodes,
            browsing_context::locate_nodes::Command {
                params: browsing_context::locate_nodes::Parameters {
                    context: browsing_context.clone(),
                    locator: browsing_context::Locator::Css(CssLocator {
                        value: r#"form[action="/openidconnect-login"] button[type="submit"]"#
                            .to_owned(),
                    }),
                    max_node_count: None,
                    ownership: None,
                    sandbox: None,
                    serialization_options: None,
                    start_nodes: None,
                },
            },
        )
        .await?;

    info!("{:?}", nodes);

    while let Ok(log) = subscription.recv().await {
        info!("received log message: {log:?}");
    }

    Ok(())
}
