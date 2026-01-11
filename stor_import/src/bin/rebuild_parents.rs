use aelita_commons::log_init;
use aelita_stor_diesel::{
    ModelJournalId, PermaStore, StorIdTypeDiesel, StorTransaction, establish_connection,
    storapi_journal_get_data,
};
use aelita_stor_diesel::{storapi_hd_parents_delete, storapi_rebuild_parents};
use aelita_stor_import::err::StorImportResult;
use std::process::ExitCode;
use xana_commons_rs::pretty_main;

fn main() -> ExitCode {
    log_init();

    pretty_main(run)
}

fn run() -> StorImportResult<()> {
    let conn = &mut establish_connection(PermaStore::AelitaNull).unwrap();

    // StorTransaction::new_transaction("truncate", conn, |conn| {
    //     // let data = storapi_journal_get_data(conn, ModelJournalId::new(1))?;
    //     // std::fs::write("journal-1.dat", data.as_inner()).unwrap();
    //     storapi_hd_parents_delete(conn)
    // })?;
    StorTransaction::new_transaction("reset", conn, |conn| storapi_rebuild_parents(conn))?;
    Ok(())
}
