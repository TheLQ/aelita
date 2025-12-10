use crate::schema::tor1_torrents;

diesel::table! {
    fast_tor_update (tor_hash) {
        #[max_length = 50]
        tor_hash -> Binary,
        #[max_length = 20]
        tor_state -> Varchar,
    }
}
pub const SQL_FAST_TOR_CREATE: &str = "CREATE TEMPORARY TABLE `fast_tor_update` (tor_hash BINARY(20) NOT NULL PRIMARY KEY, tor_state VARCHAR(20))";
pub const SQL_FAST_TOR_DROP: &str = "DROP TEMPORARY TABLE `fast_tor_update`";
diesel::joinable!(fast_tor_update -> tor1_torrents (tor_hash));
diesel::allow_tables_to_appear_in_same_query!(fast_tor_update, tor1_torrents);
