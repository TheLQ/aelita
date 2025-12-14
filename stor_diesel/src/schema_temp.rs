use crate::schema::{hd1_files_tree, tor1_torrents};

diesel::table! {
    fast_tor_update (tor_hash) {
        #[max_length = 50]
        tor_hash -> Binary,
        #[max_length = 20]
        tor_state -> Varchar,
    }
}
pub const SQL_FAST_TOR_CREATE: &str = "CREATE TEMPORARY TABLE `fast_tor_update` \
(tor_hash BINARY(20) NOT NULL PRIMARY KEY, \
tor_state VARCHAR(20) \
)";
pub const SQL_FAST_TOR_DROP: &str = "DROP TEMPORARY TABLE `fast_tor_update`";

diesel::joinable!(fast_tor_update -> tor1_torrents (tor_hash));
diesel::allow_tables_to_appear_in_same_query!(fast_tor_update, tor1_torrents);

//

diesel::table! {
    fast_hd_components (component) {
        #[max_length = 250]
        component -> Binary,
    }
}

pub const FAST_HD_COMPONENTS_CREATE: &str = "CREATE TEMPORARY TABLE `fast_hd_components` (\
`component` VARBINARY(250) NOT NULL PRIMARY KEY
)";
pub const FAST_HD_COMPONENTS_DROP: &str = "DROP TEMPORARY TABLE `fast_hd_components`";

diesel::allow_tables_to_appear_in_same_query!(fast_hd_components, hd1_files_tree);
// todo: primary keys don't match, can't override
// diesel::joinable!(fast_hd_components -> hd1_files_tree (component));

//

diesel::table! {
    fast_hd_paths (id) {
        id -> Integer,
        #[max_length = 250]
        p0 -> Nullable<Binary>,
        #[max_length = 250]
        p1 -> Nullable<Binary>,
        #[max_length = 250]
        p2 -> Nullable<Binary>,
        #[max_length = 250]
        p3 -> Nullable<Binary>,
        #[max_length = 250]
        p4 -> Nullable<Binary>,
        #[max_length = 250]
        p5 -> Nullable<Binary>,
        #[max_length = 250]
        p6 -> Nullable<Binary>,
        #[max_length = 250]
        p7 -> Nullable<Binary>,
        #[max_length = 250]
        p8 -> Nullable<Binary>,
        #[max_length = 250]
        p9 -> Nullable<Binary>,
        #[max_length = 250]
        p10 -> Nullable<Binary>,
    }
}

pub const FAST_HD_PATHS_CREATE: &str = "CREATE TEMPORARY TABLE `fast_hd_paths` (\
`id` INT AUTO_INCREMENT PRIMARY KEY,\
`p0` VARBINARY(250),\
`p1` VARBINARY(250),\
`p2` VARBINARY(250),\
`p3` VARBINARY(250),\
`p4` VARBINARY(250),\
`p5` VARBINARY(250),\
`p6` VARBINARY(250),\
`p7` VARBINARY(250),\
`p8` VARBINARY(250),\
`p9` VARBINARY(250),\
`p10` VARBINARY(250)\
)";
pub const FAST_HD_PATHS_DROP: &str = "DROP TEMPORARY TABLE `fast_hd_paths`";
