use aelita_commons::log_init;
use aelita_stor_diesel::ModelJournalImmutable;
use aelita_stor_diesel::ModelJournalTypeName;
use aelita_stor_diesel::storapi_hd_revert_by_pop;
use aelita_stor_diesel::{PermaStore, StorTransaction, establish_connection_or_panic};
use aelita_stor_diesel::{storapi_journal_commit_new, storapi_journal_commit_remain_next};
use aelita_stor_import::err::{StorImportError, StorImportResult};
use aelita_stor_import::{storcommit_hd, storcommit_torrents};
use std::ops::ControlFlow;
use std::process::ExitCode;
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{BasicWatch, pretty_main};

fn main() -> ExitCode {
    log_init();

    pretty_main(run)
}

const MEGA_TRANSACTION: bool = false;

fn run() -> StorImportResult<()> {
    let mut conn = establish_connection_or_panic(PermaStore::AelitaNull);

    StorTransaction::new_transaction("revert", &mut conn, |conn| storapi_hd_revert_by_pop(conn))?;

    let total_watch = BasicWatch::start();
    let mut total_commit = 0;
    if MEGA_TRANSACTION {
        StorTransaction::new_transaction("cli-import", &mut conn, |conn| {
            while let Some(row) = storapi_journal_commit_remain_next(conn)? {
                process_row(conn, row)?;
                total_commit += 1;
            }
            Ok::<_, StorImportError>(())
        })?;
    } else {
        loop {
            let next = StorTransaction::new_transaction("cli-import", &mut conn, |conn| {
                if let Some(row) = storapi_journal_commit_remain_next(conn)? {
                    process_row(conn, row)?;
                    total_commit += 1;
                    Ok::<_, StorImportError>(ControlFlow::Continue(()))
                } else {
                    Ok(ControlFlow::Break(()))
                }
            })?;
            if next.is_break() {
                break;
            }
        }
    }
    info!("commit {total_commit} journal in {total_watch}");

    Ok(())
}

fn process_row(conn: &mut StorTransaction, row: ModelJournalImmutable) -> StorImportResult<()> {
    let journal_id = row.journal_id.clone();
    info!("-- Commit journal {journal_id} {} --", row.journal_type);
    let journal_type = row.journal_type.clone();
    match row.journal_type {
        ModelJournalTypeName::QbGetTorJson1 => {
            // todo!()
            storcommit_torrents(conn, row)
        }
        ModelJournalTypeName::NData1 => storcommit_hd(conn, row),
        ModelJournalTypeName::Space1 => todo!(),
    }?;
    // if 1 + 1 == 2 {
    // if journal_type == ModelJournalTypeName::NData1 {
    //     panic!("no commit for you");
    // }
    storapi_journal_commit_new(conn, &journal_id)?;
    Ok(())
}
