#![feature(error_generic_member_access)]
#![feature(iterator_try_collect)]

pub mod connection;
pub mod models;

pub mod api;
pub mod err;
#[allow(unused_imports)]
pub mod schema;
pub mod tests;

pub use diesel as diesel_re;
