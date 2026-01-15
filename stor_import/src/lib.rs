#![feature(error_generic_member_access)]
#![feature(iterator_try_collect)]
#![feature(const_convert)]
#![feature(const_trait_impl)]

pub mod err;
mod importers;
mod util;

// todo: test not callable outside of main
// #[cfg(test)]
pub mod integ_test;

pub use importers::{
    impl_calls::{journal_commit, journal_commit_remain},
    n_data_v1::{fetch::storfetch_paths_from_cache, fetch::storfetch_paths_from_disk},
    qb_get_tor_json_v1::fetch::storfetch_torrents,
};
