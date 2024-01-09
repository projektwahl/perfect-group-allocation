use perfect_group_allocation_backend::error::AppError;
use perfect_group_allocation_backend::run_server;

#[tokio::main]
pub async fn main() -> Result<(), AppError> {
    let database_url = std::env::var("DATABASE_URL")?;
    run_server(&database_url).await?.await
}
