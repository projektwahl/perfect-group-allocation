use std::str::FromStr;
use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sqlx::any::AnyConnectOptions;
use sqlx::pool::PoolOptions;
use sqlx::Postgres;

use crate::error::AppError;

pub async fn get_database_connection(database_url: &str) -> Result<DatabaseConnection, AppError> {
    let mut opt = ConnectOptions::new(database_url);
    opt.connect_timeout(Duration::from_secs(3));
    Ok(Database::connect(opt).await?)
}

pub async fn get_database_connection_from_env() -> Result<DatabaseConnection, AppError> {
    let database_url = std::env::var("DATABASE_URL")?;
    get_database_connection(&database_url).await
}

pub async fn get_test_database() -> Result<DatabaseConnection, AppError> {
    get_database_connection("postgres://postgres:password@localhost:5431").await
}

pub async fn get_offline_test_database() -> Result<DatabaseConnection, AppError> {
    let mut opt = ConnectOptions::new("postgres://postgres:password@localhost-offline:5433");
    opt.connect_timeout(Duration::from_secs(3));
    opt.pool_options::<sqlx::Any>().connect_lazy_with(
        AnyConnectOptions::from_str("postgres://postgres:password@localhost-offline:5433").unwrap(),
    );
    Ok(Database::)
}
