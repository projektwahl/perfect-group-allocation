use std::sync::Arc;

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::service::Service as _;
use hyper::Request;
use perfect_group_allocation_backend::setup_server;
use perfect_group_allocation_config::{get_config, Config, OpenIdConnectConfig, TlsConfig};

#[tokio::main(flavor = "current_thread")]
pub async fn bench_client_server_function_service(value: u64) {
    let (_watcher, config) = get_config().await.unwrap();

    let service = setup_server::<Bytes>(config).unwrap();
    for _ in 0..value {
        // TODO FIXME check response
        service
            .call(
                Request::builder()
                    .uri("https://h3.selfmade4u.de")
                    .body(http_body_util::Empty::new().map_err(|error| match error {}))
                    .unwrap(),
            )
            .await
            .unwrap();
    }
}
