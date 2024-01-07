use diesel::prelude::*;

use crate::schema::project_history;

#[derive(Queryable, Selectable)]
#[diesel(table_name = project_history)]
pub struct ProjectHistoryEntry {
    id: i32,
    title: String,
}
