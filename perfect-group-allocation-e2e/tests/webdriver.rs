use perfect_group_allocation_backend::setup_http2_http3_server;
use perfect_group_allocation_config::{Config, OpenIdConnectConfig};
use tracing::info;
use webdriver_bidi::browsing_context::{self};
use webdriver_bidi::input::perform_actions::{
    Origin, PointerCommonProperties, PointerDownAction, PointerMoveAction, PointerSourceAction,
    PointerSourceActions, PointerUpAction, SourceActions,
};
use webdriver_bidi::input::ElementOrigin;
use webdriver_bidi::protocol::Extensible;
use webdriver_bidi::script::{
    ContextTarget, EvaluateResult, EvaluateResultSuccess, RemoteValue, SharedReference,
};
use webdriver_bidi::{input, script, session, Browser, SendCommand, WebDriver};

// cargo run --bin webdriver
#[tokio::main]
#[allow(clippy::too_many_lines)]
pub async fn main() -> Result<(), webdriver_bidi::Error> {
    // https://docs.docker.com/compose/production/

    // https://www.redhat.com/sysadmin/quadlet-podman
    // printf "postgrespassword" | podman secret create postgres_password -

    // the integration code should be as close as possible to production so we should use podman compose

    // podman run --rm --detach --name postgres-profiling --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres
    // podman wait --condition healthy perfect-group-allocation_postgres_1
    // podman inspect perfect-group-allocation_postgres_1

    // I think we should not start it here like that but use podman to properly start it to minimize the differences. Some lower level testing may use this here?

    let fut = setup_http2_http3_server(Config {
        url: "https://h3.selfmade4u.de".to_owned(),
        database_url: "postgres://postgres@localhost/pga?sslmode=disable".to_owned(),
        openidconnect: OpenIdConnectConfig {
            issuer_url: "http://localhost:8080/realms/pga".to_owned(),
            client_id: "pga".to_owned(),
            client_secret: "test".to_owned(),
        },
    })
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
                    url: "https://h3.selfmade4u.de".to_owned(),
                    wait: Some(browsing_context::ReadinessState::Complete),
                },
            },
        )
        .await?;

    let nodes = driver
        .send_command(
            SendCommand::ScriptEvaluate,
            script::evaluate::Command {
                params: script::evaluate::Parameters {
                    expression: r#"document.querySelector(`form[action="/openidconnect-login"] button[type="submit"]`)"#
                        .to_owned(),
                    target: script::Target::Context(ContextTarget { context: Some(browsing_context.clone()), sandbox: None }),
                    await_promise: false,
                    result_ownership: None,
                    serialization_options: None,
                    user_activation: None,
                },
            },
        )
        .await?;

    let EvaluateResult::Success(EvaluateResultSuccess {
        result: RemoteValue::Node(node),
        ..
    }) = nodes.0
    else {
        panic!();
    };
    info!("{:?}", node);

    let _result = driver
        .send_command(
            SendCommand::InputPerformActions,
            input::perform_actions::Command {
                params: input::perform_actions::Parameters {
                    context: browsing_context.clone(),
                    actions: vec![SourceActions::Pointer(PointerSourceActions {
                        id: "test".to_owned(),
                        parameters: None,
                        actions: vec![
                            PointerSourceAction::PointerMove(PointerMoveAction {
                                x: 0,
                                y: 0,
                                duration: None,
                                origin: Some(Origin::Element(ElementOrigin {
                                    element: SharedReference {
                                        shared_id: node.shared_id.unwrap().clone(),
                                        handle: node.handle.clone(),
                                        extensible: Extensible::default(),
                                    },
                                })),
                                common: PointerCommonProperties::default(),
                            }),
                            PointerSourceAction::PointerDown(PointerDownAction {
                                button: 0,
                                common: PointerCommonProperties::default(),
                            }),
                            PointerSourceAction::PointerUp(PointerUpAction {
                                button: 0,
                                common: PointerCommonProperties::default(),
                            }),
                        ],
                    })],
                },
            },
        )
        .await?;

    while let Ok(log) = subscription.recv().await {
        info!("received log message: {log:?}");
    }

    Ok(())
}
