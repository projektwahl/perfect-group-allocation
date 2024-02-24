mod error;
pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::pooled_connection::deadpool::{Object, Pool as DeadPool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
pub use error::DatabaseError;
use schema::project_history;

use crate::models::ProjectHistoryEntry;

pub type Pool = DeadPool<AsyncPgConnection>;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn get_database_connection(database_url: String) -> Result<Pool, DatabaseError> {
    let config =
        AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(&database_url);

    let pool = DeadPool::builder(config).build()?;

    // TODO do this in a once cell in case the database is not available on startup
    tokio::task::spawn_blocking(move || {
        let mut connection =
            AsyncConnectionWrapper::<AsyncPgConnection>::establish(&database_url).unwrap();
        connection.run_pending_migrations(MIGRATIONS).unwrap();
    })
    .await
    .unwrap();

    Ok(pool)
}

pub struct DatabaseConnection(pub Object<AsyncPgConnection>);

pub async fn example() -> Result<(), DatabaseError> {
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = get_database_connection(database_url).await?;

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
