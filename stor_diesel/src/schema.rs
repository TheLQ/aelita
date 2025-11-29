// @generated automatically by Diesel CLI.

diesel::table! {
    hd1_galleries (hd_id) {
        publish_id -> Unsigned<Integer>,
        space_id -> Unsigned<Integer>,
        hd_site_id -> Unsigned<Integer>,
        hd_id -> Unsigned<Integer>,
        #[max_length = 50]
        tor_hash -> Binary,
    }
}

diesel::table! {
    hd1_sites (hd_site_id) {
        publish_id -> Unsigned<Integer>,
        hd_site_id -> Unsigned<Integer>,
        #[max_length = 50]
        site_name -> Varchar,
        description -> Text,
    }
}

diesel::table! {
    journal_complete (journal_id) {
        publish_id -> Unsigned<Integer>,
        journal_id -> Unsigned<Integer>,
    }
}

diesel::table! {
    journal_data_immutable (journal_id) {
        publish_id -> Unsigned<Integer>,
        journal_id -> Unsigned<Integer>,
        journal_type -> Unsigned<Integer>,
        data -> Blob,
    }
}

diesel::table! {
    journal_data_upgraded (journal_id) {
        publish_id -> Unsigned<Integer>,
        journal_id -> Unsigned<Integer>,
        overwrites_journal_id -> Unsigned<Integer>,
        journal_type -> Unsigned<Integer>,
        data -> Blob,
    }
}

diesel::table! {
    journal_types (journal_type) {
        journal_type -> Unsigned<Integer>,
        #[max_length = 20]
        journal_type_name -> Varchar,
    }
}

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
    space (space_id) {
        publish_id -> Unsigned<Integer>,
        space_id -> Unsigned<Integer>,
        #[max_length = 50]
        space_name -> Varchar,
        description -> Text,
    }
}

diesel::table! {
    space_owned (space_id) {
        publish_id -> Unsigned<Integer>,
        space_id -> Unsigned<Integer>,
        description -> Text,
    }
}

diesel::table! {
    tor1_qb_host (qb_host_id) {
        publish_id -> Unsigned<Integer>,
        qb_host_id -> Unsigned<Integer>,
        #[max_length = 50]
        name -> Varchar,
    }
}

diesel::table! {
    tor1_status_types (tor_status_type) {
        tor_status_type -> Unsigned<Integer>,
        #[max_length = 50]
        name -> Varchar,
    }
}

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

diesel::joinable!(hd1_galleries -> hd1_sites (hd_site_id));
diesel::joinable!(hd1_galleries -> publish_log (publish_id));
diesel::joinable!(hd1_galleries -> space (space_id));
diesel::joinable!(hd1_sites -> publish_log (publish_id));
diesel::joinable!(journal_complete -> publish_log (publish_id));
diesel::joinable!(journal_data_immutable -> journal_types (journal_type));
diesel::joinable!(journal_data_immutable -> publish_log (publish_id));
diesel::joinable!(journal_data_upgraded -> journal_data_immutable (overwrites_journal_id));
diesel::joinable!(journal_data_upgraded -> journal_types (journal_type));
diesel::joinable!(journal_data_upgraded -> publish_log (publish_id));
diesel::joinable!(space -> publish_log (publish_id));
diesel::joinable!(space_owned -> publish_log (publish_id));
diesel::joinable!(space_owned -> space (space_id));
diesel::joinable!(tor1_qb_host -> publish_log (publish_id));
diesel::joinable!(tor1_torrents -> publish_log (publish_id));
diesel::joinable!(tor1_torrents -> space (space_id));

diesel::allow_tables_to_appear_in_same_query!(
    hd1_galleries,
    hd1_sites,
    journal_complete,
    journal_data_immutable,
    journal_data_upgraded,
    journal_types,
    publish_log,
    space,
    space_owned,
    tor1_qb_host,
    tor1_status_types,
    tor1_torrents,
);
