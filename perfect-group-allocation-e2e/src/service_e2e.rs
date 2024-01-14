use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::service::Service as _;
use hyper::Request;
use perfect_group_allocation_backend::config::Config;
use perfect_group_allocation_backend::setup_server;

// podman run --rm --detach --name postgres-testing --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres

// TODO FIXME use black_box

#[tokio::main(flavor = "current_thread")]
pub async fn bench_client_server_function_service(value: u64) {
    let service = setup_server::<Bytes>(Config {
        database_url: "postgres://postgres@localhost/pga?sslmode=disable".to_owned(),
        openidconnect: None,
    })
    .unwrap();
    for _ in 0..value {
        // TODO FIXME check response
        service
            .call(
                Request::builder()
                    .uri("http://localhost:3000/")
                    .body(http_body_util::Empty::new().map_err(|error| match error {}))
                    .unwrap(),
            )
            .await
            .unwrap();
    }
}
