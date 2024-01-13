use std::process::ExitCode;

use perfect_group_allocation_backend::error::AppError;
use perfect_group_allocation_backend::{run_http2_server, setup_http2_http3_server};

pub fn main() -> Result<(), AppError> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let database_url = std::env::var("DATABASE_URL")?;
            setup_http2_http3_server(database_url).await?.await
        })
}
