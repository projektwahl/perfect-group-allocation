//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "project_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub row_id: i32,
    pub id: i32,
    pub changed: String,
    pub deleted: bool,
    pub author: i32,
    pub visibility: i32,
    pub title: String,
    pub description: String,
}

#[expect(clippy::empty_enum)]
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
