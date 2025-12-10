// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(mysql_type(name = "Enum"))]
    pub struct JournalImmutableJournalTypeEnum;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(mysql_type(name = "Enum"))]
    pub struct Tor1TorrentsTorStatusEnum;
}

diesel::table! {
    hd1_galleries (hd_id) {
        journal_id -> Unsigned<Integer>,
        hd_site_id -> Unsigned<Integer>,
        hd_id -> Unsigned<Integer>,
        #[max_length = 50]
        tor_hash -> Binary,
    }
}

diesel::table! {
    hd1_sites (hd_site_id) {
        journal_id -> Unsigned<Integer>,
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
        journal_id -> Unsigned<Integer>,
        #[max_length = 13]
        journal_type -> JournalImmutableJournalTypeEnum,
        data -> Longblob,
        metadata -> Nullable<Json>,
        committed -> Bool,
        at -> Timestamp,
        #[max_length = 100]
        cause_xrn -> Nullable<Varchar>,
        cause_description -> Text,
    }
}

diesel::table! {
    space_names (space_id) {
        journal_id -> Unsigned<Integer>,
        space_id -> Unsigned<Integer>,
        #[max_length = 50]
        space_name -> Varchar,
        description -> Text,
    }
}

diesel::table! {
    space_owned (space_id) {
        journal_id -> Unsigned<Integer>,
        space_id -> Unsigned<Integer>,
        #[max_length = 100]
        child_xrn -> Varchar,
        description -> Text,
    }
}

diesel::table! {
    tor1_qb_host (qb_host_id) {
        qb_host_id -> Unsigned<Integer>,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 50]
        address -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Tor1TorrentsTorStatusEnum;

    tor1_torrents (torhash) {
        journal_id -> Unsigned<Integer>,
        #[max_length = 50]
        torhash -> Binary,
        tor_status_changed -> Timestamp,
        qb_host_id -> Unsigned<Integer>,
        #[max_length = 18]
        tor_status -> Tor1TorrentsTorStatusEnum,
    }
}

diesel::joinable!(hd1_galleries -> hd1_sites (hd_site_id));
diesel::joinable!(hd1_galleries -> journal_immutable (journal_id));
diesel::joinable!(hd1_sites -> journal_immutable (journal_id));
diesel::joinable!(space_names -> journal_immutable (journal_id));
diesel::joinable!(space_owned -> journal_immutable (journal_id));
diesel::joinable!(space_owned -> space_names (space_id));
diesel::joinable!(tor1_torrents -> journal_immutable (journal_id));

diesel::allow_tables_to_appear_in_same_query!(
    hd1_galleries,
    hd1_sites,
    journal_immutable,
    space_names,
    space_owned,
    tor1_qb_host,
    tor1_torrents,
);
