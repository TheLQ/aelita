#![feature(error_generic_member_access)]
#![feature(iterator_try_collect)]
#![feature(const_convert)]
#![feature(const_trait_impl)]

pub mod err;
mod importers;
mod path_const;
mod util;

pub use importers::{
    n_data_v1::{
        commit::storcommit_hd,
        defs::CompressedPaths,
        fetch::{COMPRESSEDD_CACHE, storfetch_ndata},
    },
    qb_get_tor_json_v1::{commit::storcommit_torrents, fetch::storfetch_torrents},
};
