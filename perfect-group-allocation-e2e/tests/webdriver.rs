use std::os::fd::{AsFd, AsRawFd};
use std::process::{exit, Stdio};

use perfect_group_allocation_backend::setup_http2_http3_server;
use perfect_group_allocation_config::{Config, OpenIdConnectConfig};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tempfile::tempdir;
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

// here
// we want to be able to run tests without starting keycloak again and again?

// so I think we should have n categories of setup (fully working, broken keycloak, crashed keycloak) that you can keep running between test setups? maybe some option to fully reset the database between runs (by restarting and deleting volume)?
// for non-CI I think we should restart our server in between runs to get code updates

#[tokio::test]
pub async fn run_test() {
    test().await.unwrap();
}

// cargo test -p perfect-group-allocation-e2e --test webdriver
#[allow(clippy::too_many_lines)]
pub async fn test() -> Result<(), webdriver_bidi::Error> {
    // podman wait --condition healthy perfect-group-allocation_postgres_1
    // podman inspect perfect-group-allocation_postgres_1

    let tmp_dir = tempdir().map_err(webdriver_bidi::ErrorInner::TmpDirCreate)?;

    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect::<String>()
        + "-";

    let base_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../deployment/kustomize/base");

    let relative_base_path = "../..".to_owned() + base_path;

    let output = tokio::process::Command::new("/usr/bin/kustomize")
        .arg("create")
        .arg("--resources")
        .arg(relative_base_path)
        .arg("--nameprefix")
        .arg(&rand_string)
        .current_dir(tmp_dir.path())
        .status()
        .await
        .unwrap();

    let kustomize_build = tokio::process::Command::new("/usr/bin/kustomize")
        .arg("build")
        .current_dir(tmp_dir.path())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let kustomize_build_stdout: Stdio = kustomize_build.stdout.unwrap().try_into().unwrap();

    // maybe we should also start webdriver inside the container by default but we should probably also support running it manually for debugging

    // podman stop --all
    // podman rm --all
    // podman volume prune
    let podman_play = tokio::process::Command::new("podman")
        .arg("kube")
        .arg("play")
        //.arg("--build")
        //.arg("--replace")
        .arg("--publish")
        .arg("8443")
        .arg("-")
        .current_dir(base_path)
        .stdin(kustomize_build_stdout)
        .stdout(Stdio::inherit())
        .status()
        .await
        .unwrap();

    println!("{:?}", tmp_dir);

    let logs = tokio::process::Command::new("podman")
        .args(["logs", "--color", "--names", "--follow"])
        .arg(format!("{rand_string}keycloak-keycloak"))
        .arg(format!("{rand_string}postgres-postgres"))
        .arg(format!("{rand_string}webdriver-pod-webdriver"))
        .status()
        .await
        .unwrap();

    // TODO FIXME cleanup

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
