// @generated automatically by Diesel CLI.

diesel::table! {
    aproject_names (xrn) {
        #[max_length = 100]
        xrn -> Varchar,
        #[max_length = 100]
        title -> Varchar,
        /// rfc3339
        #[max_length = 25]
        published -> VarChar,
    }
}

diesel::table! {
    xrn_registry (xrn) {
        #[max_length = 100]
        xrn -> Varchar,
        /// rfc3339
        #[max_length = 25]
        published -> VarChar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(aproject_names, xrn_registry,);
