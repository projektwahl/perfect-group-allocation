use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
/*
triggers have the advantage that the user does semantically more meaningful instructions like create update delete, but then the code cannot directly get e.g. the new row id etc

CREATE TABLE project_history (
    id INTEGER NOT NULL,
    changed TIMESTAMP NOT NULL,
    -- latest BOOL NOT NULL DEFAULT TRUE, -- maybe update this using trigger
    deleted BOOL NOT NULL DEFAULT FALSE,
    author INTEGER NOT NULL,
    visibility INTEGER NOT NULL, -- 0 lowest, 1 no voters, 2 no helpers, 3 no admins
    title TEXT NOT NULL,
    PRIMARY KEY (id, changed)
) WITHOUT ROWID; -- https://www.sqlite.org/withoutrowid.html

-- https://github.com/SeaQL/sea-query/pull/478
-- as long as we support mariadb and don't get performance issues we should keep this simple. but later that may be a nice way to optimize
CREATE UNIQUE INDEX project_history_index ON project_history(id) WHERE latest;
*/
// https://github.com/projektwahl/projektwahl-lit/blob/main/src/server/setup.sql
// we want to be aware of the history ids so a trigger may not be suitable
// in theory we could even model diverging history
// nice thing about only history table would be that there is only a single source of truth.
// history table can't have required fields and you can't remove fields (may not be too bad for us because we don't change the schema that often)
// we actually want to be able to roll back and not only audit so a single format may be best

// for history table indexing maybe add a latest boolean and then add a unique index to protect against wrong inserts
// and also enforce that inserts must set latest to true

// https://www.timescale.com/blog/select-the-most-recent-record-of-many-items-with-postgresql/
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(ProjectHistory::Table)
            .col(
                ColumnDef::new(ProjectHistory::RowId)
                    .integer()
                    .primary_key()
                    .auto_increment()
                    .not_null(),
            )
            .col(ColumnDef::new(ProjectHistory::Id).integer().not_null())
            .col(
                ColumnDef::new(ProjectHistory::Changed)
                    .timestamp()
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .col(
                ColumnDef::new(ProjectHistory::Deleted)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(ProjectHistory::Author)
                    .integer()
                    .default(0)
                    .not_null(),
            )
            .col(
                ColumnDef::new(ProjectHistory::Visibility)
                    .integer()
                    .default(0)
                    .not_null(),
            )
            .col(ColumnDef::new(ProjectHistory::Title).string().not_null())
            .col(
                ColumnDef::new(ProjectHistory::Description)
                    .string()
                    .not_null(),
            )
            .to_owned();

        manager.create_table(table).await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("project_history_index")
                    .table(ProjectHistory::Table)
                    .col(ProjectHistory::Id)
                    .col(ProjectHistory::Changed)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectHistory::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ProjectHistory {
    Table,
    RowId,
    Id,
    Changed,
    Deleted,
    Author,
    Visibility,
    Title,
    Description,
}
