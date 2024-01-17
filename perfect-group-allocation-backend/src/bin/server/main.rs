use perfect_group_allocation_backend::error::AppError;
use perfect_group_allocation_backend::setup_http2_http3_server;
use perfect_group_allocation_config::get_config;
use perfect_group_allocation_telemetry::setup_telemetry;

pub fn main() -> Result<(), AppError> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let _guard = setup_telemetry();

            setup_http2_http3_server(get_config()?).await?.await
        })
}
