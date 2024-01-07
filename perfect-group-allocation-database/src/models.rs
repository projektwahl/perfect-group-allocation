use diesel::prelude::*;

use crate::schema::projects_history;

#[derive(Queryable, Selectable)]
#[diesel(table_name = projects_history)]
struct Project {
    id: i32,
    title: String,
}
