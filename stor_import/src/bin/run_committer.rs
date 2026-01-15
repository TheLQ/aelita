use aelita_commons::log_init;
use aelita_stor_diesel::storapi_journal_get_journal;
use aelita_stor_diesel::{ModelJournalId, StorIdTypeDiesel, establish_connection_perma_or_panic};
use aelita_stor_diesel::{PermaStore, StorTransaction};
use aelita_stor_import::err::StorImportResult;
use aelita_stor_import::{journal_commit, journal_commit_remain};
use std::process::ExitCode;
use xana_commons_rs::pretty_main;

fn main() -> ExitCode {
    log_init();

    pretty_main(run_targeted)
}

fn run_targeted() -> StorImportResult<()> {
    let mut conn = &mut establish_connection_perma_or_panic(PermaStore::AelitaNull);

    StorTransaction::new_transaction("commit-one", &mut conn, |conn| {
        let row = storapi_journal_get_journal(conn, ModelJournalId::new(2))?;
        journal_commit(conn, row)?;
        StorImportResult::Ok(())
    })?;
    Ok(())
}

fn run() -> StorImportResult<()> {
    let conn = &mut establish_connection_perma_or_panic(PermaStore::AelitaNull);

    journal_commit_remain(conn)?;

    Ok(())
}
