// @generated automatically by Diesel CLI.

diesel::table! {
    alabel_names (xrn_label_name) {
        #[max_length = 100]
        xrn_label_name -> Varchar,
        description -> Text,
        #[max_length = 25]
        published -> Varchar,
        publish_cause -> Text,
    }
}

diesel::table! {
    aproject_lasers (xrn_laser_id) {
        xrn_laser_id -> Unsigned<Integer>,
        title -> Text,
        description -> Text,
        #[max_length = 25]
        published -> Varchar,
        publish_cause -> Text,
    }
}

diesel::table! {
    aproject_names (xrn_project_id) {
        xrn_project_id -> Unsigned<Integer>,
        title -> Text,
        description -> Text,
        #[max_length = 25]
        published -> Varchar,
        publish_cause -> Text,
    }
}

diesel::table! {
    registry_ids (xrn) {
        #[max_length = 100]
        xrn -> Varchar,
        #[max_length = 25]
        published -> Varchar,
        publish_cause -> Text,
    }
}

diesel::table! {
    registry_links (xrn_source) {
        #[max_length = 100]
        xrn_source -> Varchar,
        #[max_length = 100]
        xrn_target -> Varchar,
        #[max_length = 25]
        published -> Varchar,
        publish_cause -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    alabel_names,
    aproject_lasers,
    aproject_names,
    registry_ids,
    registry_links,
);
