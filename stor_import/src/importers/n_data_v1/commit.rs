use crate::err::{StorImportErrorKind, StorImportResult};
use aelita_stor_diesel::ModelJournalTypeName;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::storapi_hd_tree_push;
use aelita_stor_diesel::{CompressedPaths, ModelJournalImmutable};
use xana_commons_rs::ResultXanaMap;

pub fn storcommit_hd(
    conn: &mut StorTransaction,
    row: ModelJournalImmutable,
) -> StorImportResult<()> {
    assert_eq!(row.journal_type, ModelJournalTypeName::NData1);

    let compressed: CompressedPaths = row
        .data
        .deserialize_postcard()
        .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;
    storapi_hd_tree_push(conn, compressed)?;

    Ok(())
}
