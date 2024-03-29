mod error;
pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::{Object, Pool as DeadPool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
pub use error::DatabaseError;
use schema::project_history;

use crate::models::ProjectHistoryEntry;

pub type Pool = DeadPool<AsyncPgConnection>;

pub fn get_database_connection(database_url: &str) -> Result<Pool, DatabaseError> {
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    Ok(DeadPool::builder(config).build()?)
}

pub struct DatabaseConnection(pub Object<AsyncPgConnection>);

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
