// @generated automatically by Diesel CLI.

use chrono::NaiveDateTime;

diesel::table! {
    aproject_names (xrn) {
        #[max_length = 100]
        xrn -> Varchar,
        #[max_length = 100]
        title -> Varchar,
        published -> Timestamp,
    }
}

diesel::table! {
    xrn_registry (xrn) {
        #[max_length = 100]
        xrn -> Varchar,
        published -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    aproject_names,
    xrn_registry,
);
