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
    alabel_names_map (xrn_label_name) {
        #[max_length = 100]
        xrn_label_name -> Varchar,
        xrn -> Integer,
        #[max_length = 25]
        published -> Varchar,
        publish_cause -> Text,
    }
}

diesel::table! {
    aproject_names (xrn_project_id) {
        xrn_project_id -> Integer,
        title -> Text,
        description -> Text,
        #[max_length = 25]
        published -> Varchar,
    }
}

diesel::table! {
    aproject_tasks (xrn_task_id) {
        xrn_task_id -> Integer,
        title -> Text,
        description -> Text,
        #[max_length = 25]
        published -> Varchar,
        publish_cause -> Text,
    }
}

diesel::table! {
    aproject_tasks_map (xrn_task_id) {
        xrn_task_id -> Integer,
        xrn -> Integer,
        #[max_length = 25]
        published -> Varchar,
        publish_cause -> Text,
    }
}

diesel::table! {
    xrn_registry (xrn) {
        #[max_length = 100]
        xrn -> Varchar,
        #[max_length = 25]
        published -> Varchar,
    }
}

diesel::joinable!(alabel_names_map -> alabel_names (xrn_label_name));
diesel::joinable!(aproject_tasks_map -> aproject_tasks (xrn_task_id));

diesel::allow_tables_to_appear_in_same_query!(
    alabel_names,
    alabel_names_map,
    aproject_names,
    aproject_tasks,
    aproject_tasks_map,
    xrn_registry,
);
