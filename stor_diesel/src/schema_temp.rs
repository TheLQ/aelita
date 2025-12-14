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

pub const FAST_HD_COMPONENTS_CREATE: &str = "CREATE TEMPORARY TABLE IF NOT EXISTS `fast_hd_components` (\
    `component` VARBINARY(250) NOT NULL PRIMARY KEY \
    )";
pub const FAST_HD_COMPONENTS_TRUNCATE: &str = "TRUNCATE TABLE `fast_hd_components`";

diesel::allow_tables_to_appear_in_same_query!(fast_hd_components, hd1_files_tree);
// todo: primary keys don't match, can't override
// diesel::joinable!(fast_hd_components -> hd1_files_tree (component));

//

diesel::table! {
    fast_hd_paths (p0) {
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

// `id` INT AUTO_INCREMENT PRIMARY KEY,\
pub const FAST_HD_PATHS_CREATE: &str = "CREATE TEMPORARY TABLE IF NOT EXISTS `fast_hd_paths` (\
`p0` INTEGER UNSIGNED,\
`p1` INTEGER UNSIGNED,\
`p2` INTEGER UNSIGNED,\
`p3` INTEGER UNSIGNED,\
`p4` INTEGER UNSIGNED,\
`p5` INTEGER UNSIGNED,\
`p6` INTEGER UNSIGNED,\
`p7` INTEGER UNSIGNED,\
`p8` INTEGER UNSIGNED,\
`p9` INTEGER UNSIGNED,\
`p10` INTEGER UNSIGNED\
)";
pub const FAST_HD_PATHS_TRUNCATE: &str = "TRUNCATE TABLE `fast_hd_paths`";
