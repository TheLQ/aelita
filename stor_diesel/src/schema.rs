// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(mysql_type(name = "Enum"))]
    pub struct JournalImmutableJournalTypeEnum;
}

diesel::table! {
    hd1_galleries (hd_id) {
        publish_id -> Unsigned<Integer>,
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
    use diesel::sql_types::*;
    use super::sql_types::JournalImmutableJournalTypeEnum;

    journal_immutable (journal_id) {
        publish_id -> Unsigned<Integer>,
        journal_id -> Unsigned<Integer>,
        data -> Blob,
        committed -> Bool,
        #[max_length = 8]
        journal_type -> JournalImmutableJournalTypeEnum,
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
    space_names (space_id) {
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
        #[max_length = 100]
        child_xrn -> Varchar,
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
        #[max_length = 50]
        torhash -> Binary,
        tor_status_type -> Unsigned<Integer>,
        tor_status_changed -> Timestamp,
        qb_host_id -> Unsigned<Integer>,
    }
}

diesel::joinable!(hd1_galleries -> hd1_sites (hd_site_id));
diesel::joinable!(hd1_galleries -> publish_log (publish_id));
diesel::joinable!(hd1_sites -> publish_log (publish_id));
diesel::joinable!(journal_immutable -> publish_log (publish_id));
diesel::joinable!(space_names -> publish_log (publish_id));
diesel::joinable!(space_owned -> publish_log (publish_id));
diesel::joinable!(space_owned -> space_names (space_id));
diesel::joinable!(tor1_qb_host -> publish_log (publish_id));
diesel::joinable!(tor1_torrents -> publish_log (publish_id));

diesel::allow_tables_to_appear_in_same_query!(
    hd1_galleries,
    hd1_sites,
    journal_immutable,
    publish_log,
    space_names,
    space_owned,
    tor1_qb_host,
    tor1_status_types,
    tor1_torrents,
);
