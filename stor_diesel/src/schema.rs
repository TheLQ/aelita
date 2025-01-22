diesel::table! {
    xrn_registry (xrn) {
        xrn -> VarChar,
        published -> VarChar,
    }
}

diesel::table! {
    aproject_names (xrn) {
        xrn -> VarChar,
        title -> VarChar,
    }
}
