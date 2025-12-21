#![feature(error_generic_member_access)]
#![feature(iterator_try_collect)]
#![feature(slice_as_array)]
#![feature(slice_pattern)]

mod api;
mod connection;
mod err;
mod models;
pub mod path_const;
mod schema;
mod schema_temp;
pub mod tests;

pub use api::{
    api_hd::*, api_hd_mut::*, api_journal::*, api_space::*, api_tor::*, api_tor_mut::*,
    api_variables::*, show_create_table,
};
pub use connection::{
    PermaStore, StorTransaction, apply_stor_instrument, establish_connection,
    establish_connection_or_panic, load_db_url_from_env, with_quiet_sql_log_spam,
};
pub use err::{StorDieselError, StorDieselResult};
pub use models::{
    compressed_paths::*, diesel_wrappers::*, enum_types::ModelJournalTypeName, id_types::*,
    model_hd::*, model_journal::*, model_space::*, model_tor::*,
};
