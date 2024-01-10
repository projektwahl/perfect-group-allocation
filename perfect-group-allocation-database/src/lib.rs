#![feature(error_generic_member_access)]

mod error;
pub mod models;
pub mod schema;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::{Object, Pool as DeadPool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
pub use error::DatabaseError;
use http::request::Parts;
use http::StatusCode;
use schema::project_history;

use crate::models::ProjectHistoryEntry;

pub type Pool = DeadPool<AsyncPgConnection>;

pub fn get_database_connection(database_url: &str) -> Result<Pool, DatabaseError> {
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    Ok(DeadPool::builder(config).build()?)
}

pub struct DatabaseConnection(pub Object<AsyncPgConnection>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = Pool::from_ref(state);

        let conn = pool.get().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub async fn example() -> Result<(), DatabaseError> {
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = get_database_connection(&database_url)?;

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
