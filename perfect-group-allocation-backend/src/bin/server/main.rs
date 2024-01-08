use perfect_group_allocation_backend::error::AppError;
use perfect_group_allocation_backend::run_server;

#[tokio::main]
pub async fn main() -> Result<(), AppError> {
    run_server().await?.await
}
