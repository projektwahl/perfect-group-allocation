mod m20231011_000000_create_database;

use sea_orm_migration::prelude::*;

pub struct Migrator;

// https://github.com/projektwahl/projektwahl-lit/blob/main/src/server/setup.sql
// we want to be aware of the history ids so a trigger may not be suitable
// in theory we could even model diverging history
// nice thing about only history table would be that there is only a single source of truth.
// history table can't have required fields and you can't remove fields (may not be too bad for us because we don't change the schema that often)
// we actually want to be able to roll back and not only audit so a single format may be best

// for history table indexing maybe add a latest boolean and then add a unique index to protect against wrong inserts
// and also enforce that inserts must set latest to true

// https://www.timescale.com/blog/select-the-most-recent-record-of-many-items-with-postgresql/

// view or materialized view?

// where timestamp=MAX(timestamp)

/*

CREATE TABLE projects_history (
    id INTEGER NOT NULL,
    history_id INTEGER NOT NULL, -- autoincrement?
    latest BOOL NOT NULL DEFAULT TRUE,
    title TEXT NOT NULL,
    PRIMARY KEY (id, history_id)
);
CREATE UNIQUE INDEX projects_history_index ON projects_history(id) WHERE latest;


*/

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20231011_000000_create_database::Migration)]
    }
}
