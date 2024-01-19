use std::env::VarError;

use diesel_async::pooled_connection::deadpool;
use thiserror::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database url not set in env variable DATABASE_URL")]
    DatabaseEnvUrl(#[from] VarError),
    #[error("Failed to create database pool {0}")]
    PoolBuild(#[from] deadpool::BuildError),
    #[error("Database pool failed {0}")]
    Pool(#[from] deadpool::PoolError),
    #[error("Database query failed {0}")]
    Database(#[from] diesel::result::Error),
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct TestWrapper(String);
