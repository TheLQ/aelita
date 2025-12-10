use aelita_commons::log_init;
use aelita_stor_diesel::api_journal::{storapi_journal_commit_new, storapi_journal_commit_remain};
use aelita_stor_diesel::id_types::ModelJournalTypeName;
use aelita_stor_diesel::model_journal::ModelJournalDataImmutable;
use aelita_stor_diesel::{PermaStore, StorTransaction, establish_connection_or_panic};
use aelita_stor_import::err::{StorImportError, StorImportResult};
use aelita_stor_import::storcommit_torrents;
use std::process::ExitCode;
use xana_commons_rs::pretty_main;
use xana_commons_rs::tracing_re::info;

fn main() -> ExitCode {
    log_init();

    pretty_main(run)
}

const MEGA_TRANSACTION: bool = false;

fn run() -> StorImportResult<()> {
    let mut conn = establish_connection_or_panic(PermaStore::AelitaNull);

    if MEGA_TRANSACTION {
        StorTransaction::new_transaction("cli-import", &mut conn, |conn| {
            let rows = storapi_journal_commit_remain(conn)?;
            for row in rows {
                process_row(conn, row)?;
            }
            Ok::<_, StorImportError>(())
        })?;
    } else {
        let rows = StorTransaction::new_transaction("cli-import-init", &mut conn, |conn| {
            storapi_journal_commit_remain(conn)
        })?;

        for row in rows {
            StorTransaction::new_transaction("cli-import", &mut conn, |conn| {
                process_row(conn, row)
            })?;
        }
    }

    Ok(())
}

fn process_row(conn: &mut StorTransaction, row: ModelJournalDataImmutable) -> StorImportResult<()> {
    let journal_id = row.journal_id.clone();
    info!("-- Commit journal {journal_id} --");
    match row.journal_type {
        ModelJournalTypeName::QbGetTorJson1 => storcommit_torrents(conn, row),
        ModelJournalTypeName::Space1 => todo!(),
    }?;
    storapi_journal_commit_new(conn, &journal_id)?;
    Ok(())
}
