// @generated automatically by Diesel CLI.

diesel::table! {
    aproject_names (xrn_project_id) {
        xrn_project_id -> Integer,
        title -> Text,
        #[max_length = 25]
        published -> Varchar,
    }
}

diesel::table! {
    aproject_tasks (xrn_task_id) {
        xrn_task_id -> Integer,
        title -> Text,
        #[max_length = 25]
        published -> Varchar,
    }
}

diesel::table! {
    aproject_tasks_map (xrn_task_id) {
        xrn_project_id -> Integer,
        xrn_task_id -> Integer,
        #[max_length = 25]
        published -> Varchar,
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

diesel::allow_tables_to_appear_in_same_query!(
    aproject_names,
    aproject_tasks,
    aproject_tasks_map,
    xrn_registry,
);
