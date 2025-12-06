#![feature(error_generic_member_access)]
#![feature(iterator_try_collect)]
#![feature(slice_as_array)]
#![feature(slice_pattern)]

mod api;
mod common;
mod connection;
mod err;
mod example_structure;
mod models;
mod schema;
pub mod tests;

pub use api::*;
pub use common::log_init_trace;
pub use connection::{
    PermaStore, StorTransaction, establish_connection, establish_connection_or_panic,
};
pub use err::{StorDieselError, StorDieselResult};
pub use models::{id_types, model_journal, model_space, model_tor, util_types};
// pub use diesel as diesel_re;
