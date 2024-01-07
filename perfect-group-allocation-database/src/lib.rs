#![feature(error_generic_member_access)]

pub mod models;
pub mod schema;
pub mod error;

use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use error::DatabaseError;

pub async fn main() -> Result<(), DatabaseError> {
   // create a new connection pool with the default config
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(std::env::var("DATABASE_URL")?);
    let pool = Pool::builder(config).build()?;

    // checkout a connection from the pool
    let mut conn = pool.get().await?;

    // use the connection as ordinary diesel-async connection
    let res = users::table.select(User::as_select()).load::(&mut conn).await?;

    // use ordinary diesel query dsl to construct your query
    let data: Vec<User> = users::table
        .filter(users::id.gt(0))
        .or_filter(users::name.like("%Luke"))
        .select(User::as_select())
        // execute the query via the provided
        // async `diesel_async::RunQueryDsl`
        .load(&mut connection)
        .await?;

    Ok(())
}
