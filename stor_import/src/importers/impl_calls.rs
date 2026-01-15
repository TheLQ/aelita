use crate::err::{StorImportErrorKind, StorImportResult};
use crate::importers::change_op_v1::commit::storcommit_change_op_v1;
use crate::importers::n_data_v1::commit::storcommit_hd;
use crate::importers::qb_get_tor_json_v1::commit::storcommit_torrents;
use aelita_stor_diesel::{
    ModelJournalImmutable, ModelJournalTypeName, StorConnection, StorTransaction,
    storapi_journal_commit_new, storapi_journal_commit_remain_next,
};
use std::ops::ControlFlow;
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{BasicWatch, CrashErrKind};

pub fn journal_commit(
    conn: &mut StorTransaction,
    row: ModelJournalImmutable,
) -> StorImportResult<()> {
    let journal_id = row.journal_id.clone();
    info!("-- Commit journal {journal_id} {} --", row.journal_type);

    match row.journal_type {
        ModelJournalTypeName::QbGetTorJson1 => storcommit_torrents(conn, row),
        ModelJournalTypeName::NData1 => storcommit_hd(conn, row),
        ModelJournalTypeName::ChangeOp1 => storcommit_change_op_v1(conn, row),
    }?;
    storapi_journal_commit_new(conn, journal_id)?;
    Ok(())
}

const MEGA_TRANSACTION: bool = false;

pub fn journal_commit_remain(conn: &mut StorConnection) -> StorImportResult<()> {
    let total_watch = BasicWatch::start();
    let mut total_commit = 0;
    if MEGA_TRANSACTION {
        StorTransaction::new_transaction("cli-import", conn, |conn| {
            while let Some(row) = storapi_journal_commit_remain_next(conn)
                .map_err(StorImportErrorKind::DieselFailed.xana_map())?
            {
                journal_commit(conn, row)?;
                total_commit += 1;
            }
            StorImportResult::Ok(())
        })?;
    } else {
        loop {
            let next = StorTransaction::new_transaction("cli-import", conn, |conn| {
                if let Some(row) = storapi_journal_commit_remain_next(conn)
                    .map_err(StorImportErrorKind::DieselFailed.xana_map())?
                {
                    journal_commit(conn, row)?;
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
