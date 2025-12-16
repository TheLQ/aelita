use aelita_commons::log_init;
use aelita_stor_diesel::api_hd::storapi_rebuild_parents;
use aelita_stor_diesel::{PermaStore, StorTransaction, establish_connection};
use aelita_stor_import::err::StorImportResult;
use std::process::ExitCode;
use xana_commons_rs::pretty_main;

fn main() -> ExitCode {
    log_init();

    pretty_main(run)
}

fn run() -> StorImportResult<()> {
    let conn = &mut establish_connection(PermaStore::AelitaNull).unwrap();
    StorTransaction::new_transaction("reset", conn, |conn| storapi_rebuild_parents(conn))?;
    Ok(())
}
