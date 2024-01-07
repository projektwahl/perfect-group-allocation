// @generated automatically by Diesel CLI.

diesel::table! {
    project_history (history_id) {
        history_id -> Int4,
        id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 4096]
        info -> Varchar,
    }
}
