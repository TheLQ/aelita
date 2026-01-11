use aelita_commons::log_init;
use aelita_stor_diesel::ModelJournalImmutable;
use aelita_stor_diesel::storapi_hd_revert_by_pop;
use aelita_stor_diesel::{PermaStore, StorTransaction, establish_connection_or_panic};
use aelita_stor_diesel::{storapi_journal_commit_new, storapi_journal_commit_remain_next};
use aelita_stor_import::commit_journal_row;
use aelita_stor_import::err::{StorImportErrorKind, StorImportResult};
use std::ops::ControlFlow;
use std::process::ExitCode;
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{BasicWatch, CrashErrKind, pretty_main};

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
            while let Some(row) = storapi_journal_commit_remain_next(conn)
                .map_err(StorImportErrorKind::DieselFailed.xana_map())?
            {
                process_row(conn, row)?;
                total_commit += 1;
            }
            StorImportResult::Ok(())
        })?;
    } else {
        loop {
            let next = StorTransaction::new_transaction("cli-import", &mut conn, |conn| {
                if let Some(row) = storapi_journal_commit_remain_next(conn)
                    .map_err(StorImportErrorKind::DieselFailed.xana_map())?
                {
                    process_row(conn, row)?;
                    total_commit += 1;
                    StorImportResult::Ok(ControlFlow::Continue(()))
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
    commit_journal_row(conn, journal_type, row)?;
    storapi_journal_commit_new(conn, &journal_id)?;
    Ok(())
}
