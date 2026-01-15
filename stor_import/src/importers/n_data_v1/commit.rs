use crate::err::{StorImportErrorKind, StorImportResult};
use aelita_stor_diesel::ModelJournalTypeName;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::storapi_hd_tree_push;
use aelita_stor_diesel::{ModelJournalImmutable, RawDieselBytes};
use std::collections::VecDeque;
use xana_commons_rs::{CrashErrKind, ResultXanaMap};
use xana_fs_indexer_rs::CompressedPaths;

pub fn storcommit_hd(
    conn: &mut StorTransaction,
    row: ModelJournalImmutable,
) -> StorImportResult<()> {
    assert_eq!(row.journal_type, ModelJournalTypeName::NData1);

    let raw_compressed = zstd::decode_all(VecDeque::from(row.data.0)).map_err(|e| {
        StorImportErrorKind::InvalidCompressedPaths.build_message(format!("zstd failed with {e}"))
    })?;

    let compressed: CompressedPaths = RawDieselBytes(raw_compressed)
        .deserialize_postcard()
        .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;

    storapi_hd_tree_push(conn, compressed)?;

    Ok(())
}
