use crate::err::StorImportResult;
use aelita_stor_diesel::ModelJournalTypeName;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::storapi_hd_tree_push;
use aelita_stor_diesel::{CompressedPaths, ModelJournalImmutable};

pub fn storcommit_hd(
    conn: &mut StorTransaction,
    row: ModelJournalImmutable,
) -> StorImportResult<()> {
    assert_eq!(row.journal_type, ModelJournalTypeName::NData1);

    let compressed: CompressedPaths = row.data.deserialize_postcard()?;
    let paths = compressed.iter_paths().collect::<Vec<_>>();
    storapi_hd_tree_push(conn, &paths)?;

    Ok(())
}
