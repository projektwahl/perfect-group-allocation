use diesel::prelude::*;

use crate::schema::project_history;

#[derive(Queryable, Selectable)]
#[diesel(table_name = project_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectHistoryEntry {
    pub id: i32,
    pub history_id: i32,
    pub title: String,
    pub info: String,
}

#[derive(Insertable)]
#[diesel(table_name = project_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewProject {
    pub id: i32,
    pub title: String,
    pub info: String,
}
