use crate::err::StorImportResult;
use crate::importers::qb_get_tor_json_v1::defs::{ImportQbMetadata, ImportQbTorrent};
use aelita_stor_diesel::id_types::ModelJournalTypeName;
use aelita_stor_diesel::model_journal::ModelJournalDataImmutable;
use xana_commons_rs::tracing_re::info;

pub fn storcommit_torrents(row: ModelJournalDataImmutable) -> StorImportResult<()> {
    assert_eq!(row.journal_type, ModelJournalTypeName::Journal1);

    let metadata: ImportQbMetadata = row.metadata.unwrap().deserialize_json()?;
    info!("meta {metadata:?}");

    // let meta = row.data;
    // let mut meta_str = String::from_utf8(meta).unwrap();
    // meta_str.truncate(3999);
    // info!("{}", meta_str);

    let meta: Vec<ImportQbTorrent> = row.data.deserialize_json()?;

    Ok(())
}
