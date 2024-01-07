use diesel::prelude::*;

use crate::schema::project_history;

#[derive(Queryable, Selectable)]
#[diesel(table_name = project_history)]
pub struct ProjectHistoryEntry {
    pub id: i32,
    pub title: String,
}
