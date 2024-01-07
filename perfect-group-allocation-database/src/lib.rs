#![feature(error_generic_member_access)]

pub mod error;
pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use error::DatabaseError;
use schema::project_history;

use crate::models::ProjectHistoryEntry;

// https://github.com/tokio-rs/axum/tree/main/examples/diesel-async-postgres

pub fn get_database_connection(
    database_url: &str,
) -> Result<Pool<AsyncPgConnection>, DatabaseError> {
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    Ok(Pool::builder(config).build()?)
}

pub fn get_database_connection_from_env() -> Result<Pool<AsyncPgConnection>, DatabaseError> {
    let database_url = std::env::var("DATABASE_URL")?;
    get_database_connection(&database_url)
}

pub async fn example() -> Result<(), DatabaseError> {
    let pool = get_database_connection_from_env()?;

    // checkout a connection from the pool
    let mut connection = pool.get().await?;

    // use the connection as ordinary diesel-async connection
    let _res = project_history::table
        .select(ProjectHistoryEntry::as_select())
        .load(&mut connection)
        .await?;

    // use ordinary diesel query dsl to construct your query
    let _data: Vec<ProjectHistoryEntry> = project_history::table
        .filter(project_history::id.gt(0))
        .or_filter(project_history::title.like("%Luke"))
        .select(ProjectHistoryEntry::as_select())
        // execute the query via the provided
        // async `diesel_async::RunQueryDsl`
        .load(&mut connection)
        .await?;

    Ok(())
}
