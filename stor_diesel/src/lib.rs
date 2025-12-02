#![feature(error_generic_member_access)]
#![feature(iterator_try_collect)]

mod api;
mod common;
mod connection;
mod err;
mod example_structure;
mod models;
mod schema;
pub mod tests;

pub use common::log_init_trace;
pub use connection::{PermaStore, establish_connection};
pub use diesel as diesel_re;
