use sea_orm::sea_query::extension::postgres::TypeCreateStatement;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20231011_000000_create_database"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /*
    triggers have the advantage that the user does semantically more meaningful instructions like create update delete

    CREATE TABLE project_history (
        id INTEGER NOT NULL,
        changed TIMESTAMP NOT NULL,
        latest BOOL NOT NULL DEFAULT TRUE,
        deleted BOOL NOT NULL DEFAULT FALSE,
        author INTEGER NOT NULL,
        visibility INTEGER NOT NULL, -- 0 lowest, 1 no voters, 2 no helpers, 3 no admins
        title TEXT NOT NULL,
        PRIMARY KEY (id, changed)
    ) WITHOUT ROWID; -- https://www.sqlite.org/withoutrowid.html
    CREATE UNIQUE INDEX project_history_index ON project_history(id) WHERE latest;
    */
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectHistory::Table)
                    .col(ColumnDef::new(ProjectHistory::Id).integer().not_null())
                    .col(ColumnDef::new(ProjectHistory::Changed).integer().not_null())
                    .col(
                        ColumnDef::new(ProjectHistory::Latest)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(ProjectHistory::Deleted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(ProjectHistory::Author).integer().not_null())
                    .col(
                        ColumnDef::new(ProjectHistory::Visibility)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectHistory::Title).string().not_null())
                    .col(
                        ColumnDef::new(ProjectHistory::Description)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectHistory::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ProjectHistory {
    Table,
    Id,
    Changed,
    Latest,
    Deleted,
    Author,
    Visibility,
    Title,
    Description,
}
