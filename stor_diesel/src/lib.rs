#![feature(error_generic_member_access)]
#![feature(iterator_try_collect)]

pub mod api;
pub mod common;
pub mod connection;
pub mod err;
mod example_structure;
pub mod models;
#[allow(unused_imports)]
pub mod schema;
pub mod tests;

pub use diesel as diesel_re;
