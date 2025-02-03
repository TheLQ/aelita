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
    fire_history (xrn_firehist_id) {
        xrn_firehist_id -> Unsigned<Integer>,
        #[max_length = 13]
        fire_id -> Varchar,
        fire_last_visit -> Unsigned<Integer>,
        visit_count -> Unsigned<Integer>,
        title -> Text,
        #[max_length = 25]
        published -> Varchar,
        publish_cause -> Text,
    }
}

diesel::table! {
    jnl_id_counters (key) {
        #[max_length = 40]
        key -> Varchar,
        counter -> Unsigned<Integer>,
        #[max_length = 25]
        updated -> Varchar,
    }
}

diesel::table! {
    jnl_mutation (mut_id) {
        mut_id -> Unsigned<Integer>,
        #[max_length = 40]
        mut_type -> Varchar,
        data -> Text,
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
    fire_history,
    jnl_id_counters,
    jnl_mutation,
    registry_ids,
    registry_links,
);
