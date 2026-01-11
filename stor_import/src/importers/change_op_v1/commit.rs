use crate::err::{StorImportErrorKind, StorImportResult};
use crate::importers::change_op_v1::changer::{ChangeOp, Changer};
use aelita_stor_diesel::{ModelJournalImmutable, StorTransaction};
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{BasicWatch, CrashErrKind, ResultXanaMap};

pub fn storcommit_change_op_v1(
    conn: &mut StorTransaction,
    row: ModelJournalImmutable,
) -> StorImportResult<()> {
    let changes: Vec<ChangeOp> = row
        .data
        .deserialize_json()
        .map_err(|(e, m)| StorImportErrorKind::InvalidChangeOp.build_err_message(e, m))?;
    let watch = BasicWatch::start();
    let changes_len = changes.len();
    for change in changes {
        change.commit_change(conn)?;
    }
    info!("Committed {changes_len} change ops in {watch}");

    Ok(())
}
