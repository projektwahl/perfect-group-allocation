use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::project_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectHistoryEntry {
    pub id: i32,
    pub history_id: i32,
    pub title: String,
    pub info: String,
}
