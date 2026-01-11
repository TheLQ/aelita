use crate::err::{StorImportErrorKind, StorImportResult};
use crate::importers::change_op_v1::changer::{ChangeOp, Changer};
use aelita_stor_diesel::{ModelJournalImmutable, StorTransaction};
use xana_commons_rs::{CrashErrKind, ResultXanaMap};

pub fn storcommit_change_op_v1(
    conn: &mut StorTransaction,
    row: ModelJournalImmutable,
) -> StorImportResult<()> {
    let change: ChangeOp = row
        .data
        .deserialize_json()
        .map_err(|(e, m)| StorImportErrorKind::InvalidChangeOp.build_err_message(e, m))?;
    change.commit_change(conn)?;

    Ok(())
}
