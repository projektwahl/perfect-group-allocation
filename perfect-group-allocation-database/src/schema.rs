// @generated automatically by Diesel CLI.

diesel::table! {
    project_history (history_id) {
        history_id -> Int4,
        id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 4096]
        info -> Varchar,
        #[max_length = 256]
        place -> Varchar,
        costs -> Float8,
        min_age -> Int4,
        max_age -> Int4,
        min_participants -> Int4,
        max_participants -> Int4,
        random_assignments -> Bool,
        deleted -> Bool,
        last_updated_by -> Nullable<Int4>,
    }
}
