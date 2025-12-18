use crate::schema::{hd1_files_components, tor1_torrents};

diesel::table! {
    fast_tor_update (tor_hash) {
        #[max_length = 50]
        tor_hash -> Binary,
        #[max_length = 20]
        tor_state -> Varchar,
    }
}
pub const SQL_FAST_TOR_CREATE: &str = "CREATE TEMPORARY TABLE IF NOT EXISTS `fast_tor_update` \
(tor_hash BINARY(20) NOT NULL PRIMARY KEY, \
tor_state VARCHAR(20) \
)";
pub const SQL_FAST_TOR_TRUNCATE: &str = "TRUNCATE TABLE `fast_tor_update`";

diesel::joinable!(fast_tor_update -> tor1_torrents (tor_hash));
diesel::allow_tables_to_appear_in_same_query!(fast_tor_update, tor1_torrents);

//

diesel::table! {
    fast_hd_components (component) {
        #[max_length = 250]
        component -> Binary,
    }
}

pub const FAST_HD_COMPONENTS_CREATE: &str = "CREATE TEMPORARY TABLE IF NOT EXISTS `fast_hd_components` (\
    `component` VARBINARY(250) NOT NULL PRIMARY KEY \
    )";
pub const FAST_HD_COMPONENTS_TRUNCATE: &str = "TRUNCATE TABLE `fast_hd_components`";

diesel::allow_tables_to_appear_in_same_query!(fast_hd_components, hd1_files_components);
// todo: primary keys don't match, can't override
// diesel::joinable!(fast_hd_components -> hd1_files_tree (component));
