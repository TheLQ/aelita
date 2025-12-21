// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(mysql_type(name = "Enum"))]
    pub struct JournalImmutableJournalTypeEnum;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(mysql_type(name = "Enum"))]
    pub struct SpaceOwnedChildType1Enum;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(mysql_type(name = "Enum"))]
    pub struct SpaceOwnedChildType2Enum;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(mysql_type(name = "Enum"))]
    pub struct Tor1TorrentsStateEnum;
}

diesel::table! {
    hd1_files_components (id) {
        id -> Unsigned<Integer>,
        #[max_length = 250]
        component -> Varbinary,
    }
}

diesel::table! {
    hd1_files_parents (tree_id) {
        tree_id -> Unsigned<Integer>,
        tree_depth -> Unsigned<Integer>,
        component_id -> Unsigned<Integer>,
        parent_id -> Nullable<Unsigned<Integer>>,
    }
}

diesel::table! {
    hd1_files_parents_bak (tree_id) {
        tree_id -> Unsigned<Integer>,
        tree_depth -> Unsigned<Integer>,
        component_id -> Unsigned<Integer>,
        parent_id -> Nullable<Unsigned<Integer>>,
    }
}

diesel::table! {
    hd1_files_paths (path_id) {
        path_id -> Unsigned<Integer>,
        p0 -> Nullable<Unsigned<Integer>>,
        p1 -> Nullable<Unsigned<Integer>>,
        p2 -> Nullable<Unsigned<Integer>>,
        p3 -> Nullable<Unsigned<Integer>>,
        p4 -> Nullable<Unsigned<Integer>>,
        p5 -> Nullable<Unsigned<Integer>>,
        p6 -> Nullable<Unsigned<Integer>>,
        p7 -> Nullable<Unsigned<Integer>>,
        p8 -> Nullable<Unsigned<Integer>>,
        p9 -> Nullable<Unsigned<Integer>>,
        p10 -> Nullable<Unsigned<Integer>>,
    }
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
        metadata -> Nullable<Json>,
        committed -> Bool,
        at -> Timestamp,
        #[max_length = 100]
        cause_xrn -> Nullable<Varchar>,
        cause_description -> Text,
    }
}

diesel::table! {
    journal_immutable_data (journal_id) {
        journal_id -> Unsigned<Integer>,
        data -> Longblob,
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
    use diesel::sql_types::*;
    use super::sql_types::SpaceOwnedChildType1Enum;
    use super::sql_types::SpaceOwnedChildType2Enum;

    space_owned (ref_id) {
        ref_id -> Unsigned<Integer>,
        journal_id -> Unsigned<Integer>,
        space_id -> Unsigned<Integer>,
        #[max_length = 5]
        child_type1 -> SpaceOwnedChildType1Enum,
        #[max_length = 6]
        child_type2 -> SpaceOwnedChildType2Enum,
        child_id -> Unsigned<Integer>,
        description -> Nullable<Text>,
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
    use super::sql_types::Tor1TorrentsStateEnum;

    tor1_torrents (infohash_v1) {
        journal_id -> Unsigned<Integer>,
        qb_host_id -> Unsigned<Integer>,
        #[max_length = 20]
        infohash_v1 -> Binary,
        #[max_length = 32]
        infohash_v2 -> Binary,
        name -> Text,
        comment -> Text,
        path -> Text,
        progress -> Float,
        original_size -> Nullable<Unsigned<Bigint>>,
        selected_size -> Nullable<Unsigned<Bigint>>,
        downloaded -> Unsigned<Bigint>,
        uploaded -> Unsigned<Bigint>,
        secs_active -> Unsigned<Integer>,
        secs_seeding -> Unsigned<Integer>,
        added_on -> Timestamp,
        completion_on -> Nullable<Timestamp>,
        #[max_length = 18]
        state -> Tor1TorrentsStateEnum,
    }
}

diesel::joinable!(hd1_galleries -> hd1_sites (hd_site_id));
diesel::joinable!(hd1_galleries -> journal_immutable (journal_id));
diesel::joinable!(hd1_sites -> journal_immutable (journal_id));
diesel::joinable!(journal_immutable_data -> journal_immutable (journal_id));
diesel::joinable!(space_names -> journal_immutable (journal_id));
diesel::joinable!(space_owned -> journal_immutable (journal_id));
diesel::joinable!(space_owned -> space_names (space_id));
diesel::joinable!(tor1_torrents -> journal_immutable (journal_id));

diesel::allow_tables_to_appear_in_same_query!(
    hd1_files_components,
    hd1_files_parents,
    hd1_files_parents_bak,
    hd1_files_paths,
    hd1_galleries,
    hd1_sites,
    journal_immutable,
    journal_immutable_data,
    space_names,
    space_owned,
    tor1_qb_host,
    tor1_torrents,
);
