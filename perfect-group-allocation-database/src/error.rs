use std::backtrace::Backtrace;
use std::env::VarError;

use diesel_async::pooled_connection::deadpool;
use thiserror::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database url not set in env variable DATABASE_URL")]
    DatabaseEnvUrl(#[from] VarError, Backtrace),
    #[error("Failed to create database pool {0}\n{1}")]
    PoolBuild(#[from] deadpool::BuildError, Backtrace),
    #[error("Database pool failed {0}\n{1}")]
    Pool(#[from] deadpool::PoolError, Backtrace),
    #[error("Database query failed {0}\n{1}")]
    Database(#[from] diesel::result::Error, Backtrace),
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct TestWrapper(String);
