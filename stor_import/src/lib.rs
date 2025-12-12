#![feature(error_generic_member_access)]
#![feature(iterator_try_collect)]

pub mod err;
mod importers;
mod util;

pub use importers::{
    n_data_v1::fetch::storfetch_ndata,
    qb_get_tor_json_v1::{commit::storcommit_torrents, fetch::storfetch_torrents},
};
