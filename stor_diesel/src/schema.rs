// @generated automatically by Diesel CLI.

diesel::table! {
    aproject_names (xrn) {
        #[max_length = 100]
        xrn -> Varchar,
        #[max_length = 100]
        title -> Varchar,
    }
}

diesel::table! {
    xrn_registry (xrn) {
        #[max_length = 100]
        xrn -> Varchar,
        #[max_length = 100]
        published -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    aproject_names,
    xrn_registry,
);
