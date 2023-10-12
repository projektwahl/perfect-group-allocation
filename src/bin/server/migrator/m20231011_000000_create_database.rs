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
    CREATE TABLE project_history (
        id INTEGER NOT NULL,
        history_id INTEGER NOT NULL, -- could be the same for multiple ids if the action is atomic?
        latest BOOL NOT NULL DEFAULT TRUE,
        author INTEGER NOT NULL,
        deleted BOOL NOT NULL DEFAULT FALSE,
        visibility INTEGER NOT NULL, -- 0 lowest, 1 no voters, 2 no helpers, 3 no admins
        title TEXT NOT NULL,
        PRIMARY KEY (id, history_id)
    );
    CREATE UNIQUE INDEX project_history_index ON project_history(id) WHERE latest;
    */
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectHistory::Table)
                    .col(ColumnDef::new(ProjectHistory::Id).integer().not_null())
                    .col(
                        ColumnDef::new(ProjectHistory::HistoryId)
                            .integer()
                            .not_null(),
                    )
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
                    .col(ColumnDef::new(ProjectHistory::Name).string().not_null())
                    .col(
                        ColumnDef::new(ProjectHistory::ProfitMargin)
                            .double()
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
    HistoryId,
    Latest,
    Deleted,
    Title,
}
