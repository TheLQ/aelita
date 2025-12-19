use crate::err::StorImportResult;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::api_hd_mut::storapi_hd_tree_push;
use aelita_stor_diesel::compressed_paths::CompressedPaths;
use aelita_stor_diesel::id_types::ModelJournalTypeName;
use aelita_stor_diesel::model_journal::ModelJournalImmutable;

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
