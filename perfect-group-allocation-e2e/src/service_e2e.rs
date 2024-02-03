use std::sync::Arc;

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::service::Service as _;
use hyper::Request;
use perfect_group_allocation_backend::setup_server;
use perfect_group_allocation_config::{Config, OpenIdConnectConfig, TlsConfig};

// podman run --rm --detach --name postgres-testing --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres

// TODO FIXME use black_box

#[tokio::main(flavor = "current_thread")]
pub async fn bench_client_server_function_service(value: u64) {
    let (tx, rx) = tokio::sync::watch::channel(Arc::new(Config {
        url: "https://h3.selfmade4u.de".to_owned(),
        database_url: "postgres://postgres@localhost/pga?sslmode=disable".to_owned(),
        openidconnect: OpenIdConnectConfig {
            issuer_url: "http://localhost:8080/realms/pga".to_owned(),
            client_id: "pga".to_owned(),
            client_secret: "test".to_owned(),
        },
        // TODO FIXME generate test certificates
        tls: TlsConfig {
            cert: todo!(),
            key: todo!(),
        },
    }));

    let service = setup_server::<Bytes>(rx).unwrap();
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
