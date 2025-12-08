use aelita_stor_diesel::api_journal::storapi_journal_commit_remain;
use aelita_stor_diesel::{PermaStore, StorTransaction, establish_connection_or_panic};
use aelita_stor_import::common::log_init_trace;
use aelita_stor_import::err::{StorImportError, StorImportResult};
use aelita_stor_import::{storcommit_torrents, storfetch_torrents};
use std::process::ExitCode;
use xana_commons_rs::pretty_main;

fn main() -> ExitCode {
    log_init_trace();

    pretty_main(run)
}

fn run() -> StorImportResult<()> {
    let mut conn = establish_connection_or_panic(PermaStore::AelitaNull);

    StorTransaction::new_transaction("cli-import", &mut conn, |conn| {
        let rows = storapi_journal_commit_remain(conn)?;
        for row in rows {
            storcommit_torrents(row)?;
        }
        Ok::<_, StorImportError>(())
    })?;

    Ok(())
}
