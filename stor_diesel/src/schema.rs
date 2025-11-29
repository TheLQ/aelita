//
// Injest
//

diesel::table! {
    publish_log (publish_id) {
        publish_id -> Unsigned<Integer>,
        at -> Timestamp,
        #[max_length = 100]
        cause_xrn -> Nullable<Varchar>,
        cause_description -> Text,
    }
}

diesel::table! {
    injest_types (injest_type) {
        injest_type -> Unsigned<Integer>,
        #[max_length = 20]
        name -> Varchar,
    }
}

//

diesel::table! {
    injest_data_immutable (injest_id) {
        publish_id -> Unsigned<Integer>,
        injest_id -> Unsigned<Integer>,
        injest_type -> Unsigned<Integer>,
        data -> Blob,
    }
}
diesel::joinable!(injest_data_immutable -> publish_log (publish_id));
// diesel::allow_tables_to_appear_in_same_query!(injest_data_immutable, publish_log);

diesel::table! {
    injest_data_upgraded (injest_id) {
        publish_id -> Unsigned<Integer>,
        injest_id -> Unsigned<Integer>,
        injest_type -> Unsigned<Integer>,
        data -> Blob,
    }
}
diesel::joinable!(injest_data_upgraded -> publish_log (publish_id));
// diesel::allow_tables_to_appear_in_same_query!(injest_data_upgraded, publish_log);

diesel::table! {
    injest_complete (injest_id) {
        publish_id -> Unsigned<Integer>,
        injest_id -> Unsigned<Integer>,
    }
}
diesel::joinable!(injest_complete -> publish_log (publish_id));
diesel::joinable!(injest_complete -> injest_data_immutable (injest_id));
diesel::joinable!(injest_complete -> injest_data_upgraded (injest_id));
// diesel::allow_tables_to_appear_in_same_query!(
//     injest_complete,
//     publish_log,
//     injest_data_immutable,
//     injest_data_upgraded
// );

//
// Space
//

diesel::table! {
    space (space_id) {
        publish_id -> Unsigned<Integer>,
        space_id -> Unsigned<Integer>,
        #[max_length = 50]
        project_name -> Varchar,
        description -> Text,
    }
}
diesel::joinable!(space -> publish_log (publish_id));

diesel::table! {
    space_owned (space_id) {
        publish_id -> Unsigned<Integer>,
        space_id -> Unsigned<Integer>,
        description -> Text,
    }
}
diesel::joinable!(space_owned -> publish_log (publish_id));

//
// hd1
//

diesel::table! {
    hd1_sites (hd_site_id) {
        hd_site_id -> Unsigned<Integer>,
        #[max_length = 50]
        site_name -> Varchar,
        description -> Text,
    }
}

diesel::table! {
    hd1_galleries (hd_id) {
        publish_id -> Unsigned<Integer>,
        project_id -> Unsigned<Integer>,
        hd_site_id -> Unsigned<Integer>,
        hd_id -> Unsigned<Integer>,
        #[max_length = 50]
        tor_hash -> Binary,
    }
}
diesel::joinable!(hd1_galleries -> publish_log (publish_id));
diesel::joinable!(hd1_galleries -> hd1_sites (hd_site_id));

//
// tor
//

diesel::table! {
    tor1_status_types (publish_id) {
        tor_status_type -> Unsigned<Integer>,
        #[max_length = 50]
        name -> Varchar
    }
}

diesel::table! {
    tor1_qb_host (publish_id) {
        qb_host_id -> Unsigned<Integer>,
        #[max_length = 50]
        name -> Varchar
    }
}

//

diesel::table! {
    tor1_torrents (torhash) {
        publish_id -> Unsigned<Integer>,
        space_id -> Unsigned<Integer>,
        #[max_length = 50]
        torhash -> Binary,
        tor_status_type -> Unsigned<Integer>,
        tor_status_changed -> Timestamp,
        qb_host_id -> Unsigned<Integer>,
    }
}

//
// Other
//

diesel::allow_tables_to_appear_in_same_query!(
    injest_complete,
    publish_log,
    injest_data_immutable,
    injest_data_upgraded
);
