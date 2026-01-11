use crate::err::StorImportResult;
use crate::importers::change_op_v1::commit::storcommit_change_op_v1;
use crate::importers::n_data_v1::commit::storcommit_hd;
use crate::importers::qb_get_tor_json_v1::commit::storcommit_torrents;
use aelita_stor_diesel::{ModelJournalImmutable, ModelJournalTypeName, StorTransaction};

pub fn commit_journal_row(
    conn: &mut StorTransaction,
    journal_type: ModelJournalTypeName,
    row: ModelJournalImmutable,
) -> StorImportResult<()> {
    match journal_type {
        ModelJournalTypeName::QbGetTorJson1 => storcommit_torrents(conn, row),
        ModelJournalTypeName::NData1 => storcommit_hd(conn, row),
        ModelJournalTypeName::ChangeOp1 => storcommit_change_op_v1(conn, row),
    }
}
