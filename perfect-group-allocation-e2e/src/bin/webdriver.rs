use std::panic::AssertUnwindSafe;

use futures_util::FutureExt;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::json;
use tracing::info;
use webdriver_bidi::browsing_context::{self, BrowsingContext};
use webdriver_bidi::input::perform_actions::{
    KeyDownAction, KeySourceAction, KeySourceActions, Origin, PointerCommonProperties,
    PointerDownAction, PointerMoveAction, PointerSourceAction, PointerSourceActions,
    PointerUpAction, SourceActions,
};
use webdriver_bidi::input::ElementOrigin;
use webdriver_bidi::protocol::Extensible;
use webdriver_bidi::script::{
    ContextTarget, EvaluateResult, EvaluateResultSuccess, NodeRemoteValue, RemoteValue,
    SerializationOptions, SharedReference,
};
use webdriver_bidi::session::CapabilityRequest;
use webdriver_bidi::{input, script, session, Browser, SendCommand, WebDriver};

// here
// we want to be able to run tests without starting keycloak again and again?

// so I think we should have n categories of setup (fully working, broken keycloak, crashed keycloak) that you can keep running between test setups? maybe some option to fully reset the database between runs (by restarting and deleting volume)?
// for non-CI I think we should restart our server in between runs to get code updates

#[tokio::main]
pub async fn main() {
    let result = AssertUnwindSafe(test()).catch_unwind().await;
    println!("{result:?}");
    match result {
        Err(err) => {
            let netlog = tokio::fs::read_to_string("/tmp/netlog.json").await.unwrap();
            println!("{netlog}");
            println!("error {err:?}");
            panic!("{err:?}");
        }
        Ok(Err(err)) => {
            let netlog = tokio::fs::read_to_string("/tmp/netlog.json").await.unwrap();
            println!("{netlog}");
            println!("error {err:?}");
            panic!("{err:?}");
        }
        Ok(Ok(())) => {}
    }
}

// ./run-integration-tests.sh keycloak # once at some point
// ./run-integration-tests.sh prepare # after every recompile
// cargo test -p perfect-group-allocation-e2e --test webdriver
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub async fn test() -> Result<(), webdriver_bidi::Error> {
    tracing_subscriber::fmt::init();

    // TODO FIXME get url
    let url = "https://eperfect-group-allocation".to_string();

    // TODO FIXME add network slowdown for testing
    // TODO FIXME use user contexts for cookie isolation

    let driver = WebDriver::new(Browser::Chromium).await?;
    let _session = driver
        .send_command(
            SendCommand::SessionNew,
            session::new::Command {
                params: session::new::Parameters {
                    capabilities: session::CapabilitiesRequest {
                        // https://developer.mozilla.org/en-US/docs/Web/WebDriver/Capabilities/firefoxOptions
                        // https://chromedriver.chromium.org/capabilities
                        always_match: Some(CapabilityRequest {
                            browser_name: Some("chrome".to_owned()), // Some("firefox".to_owned()),
                            extensible: Extensible(
                                json!({
                                    "goog:chromeOptions": {
                                        "args": ["--headless", "--log-net-log=/tmp/netlog.json", "--ozone-platform=wayland"]
                                    }
                                })
                                .as_object()
                                .unwrap()
                                .to_owned(),
                            ),
                            /*extensible: Extensible(
                                json!({
                                    "moz:firefoxOptions": {
                                    //"log": {"level": "trace"}
                                    }
                                })
                                .as_object()
                                .unwrap()
                                .to_owned(),
                            ),*/
                            accept_insecure_certs: Some(false),
                            browser_version: None,
                            platform_name: None,
                            proxy: None,
                            web_socket_url: None,
                        }),
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
            SendCommand::SubscribeGlobalBrowsingContextLoad,
            Some(browsing_context.clone()),
        )
        .await?;
    let _navigation = driver
        .send_command(
            SendCommand::BrowsingContextNavigate,
            browsing_context::navigate::Command {
                params: browsing_context::navigate::Parameters {
                    context: browsing_context.clone(),
                    url,
                    wait: Some(browsing_context::ReadinessState::Complete),
                },
            },
        )
        .await?;

    let node = find_element(
        &driver,
        &browsing_context,
        r#"form[action="/openidconnect-login"] button[type="submit"]"#,
    )
    .await?;

    while let Ok(load) = subscription.try_recv() {
        info!("before: {load:?}");
    }

    click(&driver, &browsing_context, &node).await?;

    // seems like this is not sufficient
    let Ok(load) = subscription.recv().await else {
        panic!("failed")
    };

    info!("page loaded: {load:?}");

    let username = find_element(&driver, &browsing_context, "#username").await?;
    info!("{:?}", username);

    let password = find_element(&driver, &browsing_context, "#password").await?;
    info!("{:?}", password);

    click(&driver, &browsing_context, &username).await?;
    send_keys(&driver, &browsing_context, "test").await?;
    click(&driver, &browsing_context, &password).await?;
    send_keys(&driver, &browsing_context, "test").await?;

    let login_button = find_element(&driver, &browsing_context, "#kc-login").await?;
    click(&driver, &browsing_context, &login_button).await?;

    let Ok(load) = subscription.recv().await else {
        panic!("failed")
    };

    info!("page loaded: {load:?}");

    let logout_button = find_element(&driver, &browsing_context, "#logout-button").await?;

    info!("{:#?}", logout_button);

    let children = logout_button.value.unwrap().children.unwrap();
    let text = children[0]
        .inner
        .value
        .as_ref()
        .unwrap()
        .node_value
        .as_ref()
        .unwrap();

    assert_eq!(text, "Logout test@example.com");

    Ok(())
}

pub async fn find_element(
    driver: &WebDriver,
    browsing_context: &BrowsingContext,
    css_selector: &str,
) -> Result<NodeRemoteValue, webdriver_bidi::Error> {
    let nodes = driver
        .send_command(
            SendCommand::ScriptEvaluate,
            script::evaluate::Command {
                params: script::evaluate::Parameters {
                    expression: format!("document.querySelector(`{css_selector}`)"), // TODO FIXME XSS, maybe use pre
                    target: script::Target::Context(ContextTarget {
                        context: Some(browsing_context.clone()),
                        sandbox: None,
                    }),
                    await_promise: false,
                    result_ownership: None,
                    serialization_options: Some(SerializationOptions {
                        max_dom_depth: Some(1),
                        max_object_depth: None,
                        include_shadow_tree: None,
                    }),
                    user_activation: None,
                },
            },
        )
        .await?;

    if let EvaluateResult::Success(EvaluateResultSuccess {
        result: RemoteValue::Node(node),
        ..
    }) = nodes.0
    {
        Ok(node)
    } else {
        Err(webdriver_bidi::Error::ElementNotFound(
            css_selector.to_owned(),
        ))
    }
}

pub async fn send_keys(
    driver: &WebDriver,
    browsing_context: &BrowsingContext,
    text: &str,
) -> Result<(), webdriver_bidi::Error> {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect::<String>();

    let _result = driver
        .send_command(
            SendCommand::InputPerformActions,
            input::perform_actions::Command {
                params: input::perform_actions::Parameters {
                    context: browsing_context.clone(),
                    actions: vec![SourceActions::Key(KeySourceActions {
                        id: rand_string,
                        actions: text
                            .chars()
                            .flat_map(|c| {
                                vec![KeySourceAction::KeyDown(KeyDownAction {
                                    value: c.to_string(),
                                })]
                            })
                            .collect(),
                    })],
                },
            },
        )
        .await?;
    Ok(())
}

pub async fn click(
    driver: &WebDriver,
    browsing_context: &BrowsingContext,
    node: &NodeRemoteValue,
) -> Result<(), webdriver_bidi::Error> {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect::<String>();

    let _result = driver
        .send_command(
            SendCommand::InputPerformActions,
            input::perform_actions::Command {
                params: input::perform_actions::Parameters {
                    context: browsing_context.clone(),
                    actions: vec![SourceActions::Pointer(PointerSourceActions {
                        id: rand_string,
                        parameters: None,
                        actions: vec![
                            PointerSourceAction::PointerMove(PointerMoveAction {
                                x: 0,
                                y: 0,
                                duration: None,
                                origin: Some(Origin::Element(ElementOrigin {
                                    element: SharedReference {
                                        shared_id: node.shared_id.as_ref().unwrap().clone(),
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

    Ok(())
}
