use std::backtrace::Backtrace;

use diesel_async::pooled_connection::deadpool;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{inner}\n{backtrace}")]
pub struct Error {
    #[from]
    inner: DatabaseErrorInner,
    backtrace: Backtrace,
}

#[derive(Error, Debug)]
pub enum DatabaseErrorInner {
    #[error("Database url not set in env variable DATABASE_URL")]
    DatabaseEnvUrl,
    #[error("Database pool failed {0}")]
    Deadpool(#[from] deadpool::BuildError),
}
