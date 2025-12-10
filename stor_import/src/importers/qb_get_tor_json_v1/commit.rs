use crate::err::StorImportResult;
use crate::importers::qb_get_tor_json_v1::defs::{ImportQbMetadata, ImportQbTorrent};
use crate::util::HashExtractor;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::api_tor::{
    storapi_tor_torrents_get_by_hash, storapi_tor_torrents_new, storapi_tor_torrents_update_status,
};
use aelita_stor_diesel::id_types::ModelJournalTypeName;
use aelita_stor_diesel::model_journal::ModelJournalDataImmutable;
use aelita_stor_diesel::model_tor::NewModelTorrents;
use xana_commons_rs::tracing_re::info;

pub fn storcommit_torrents(
    conn: &mut StorTransaction,
    row: ModelJournalDataImmutable,
) -> StorImportResult<()> {
    assert_eq!(row.journal_type, ModelJournalTypeName::QbGetTorJson1);

    let metadata: ImportQbMetadata = row.metadata.unwrap().deserialize_json()?;
    info!("meta {metadata:?}");

    // let meta = row.data;
    // let mut meta_str = String::from_utf8(meta).unwrap();
    // meta_str.truncate(3999);
    // info!("{}", meta_str);

    let local_tors_raw: Vec<ImportQbTorrent> = row.data.deserialize_json()?;
    let local_tors = local_tors_raw.as_tor_lookup_by_hash();

    let db_tors_raw = storapi_tor_torrents_get_by_hash(
        conn,
        local_tors
            .keys()
            .map(|v| (*v).into())
            .collect::<Vec<_>>()
            .as_slice(),
    )?;
    let db_tors = db_tors_raw.as_tor_lookup_by_hash();

    let mut new_torrents = Vec::new();
    for (local_hash, local_tor) in local_tors {
        if let Some(model_tor) = db_tors.get(local_hash) {
            if *model_tor.tor_status.inner() == local_tor.state {
                // don't update anything
                continue;
            }
            storapi_tor_torrents_update_status(conn, &local_hash, &local_tor.state)?;
        } else {
            new_torrents.push(NewModelTorrents {
                journal_id: row.journal_id,
                torhash: local_hash.into(),
                qb_host_id: metadata.qb_host_id,
                tor_status: local_tor.state.into(),
            })
        }
    }
    info!("inserting {} new torrents", new_torrents.len());
    storapi_tor_torrents_new(conn, &new_torrents)?;

    Ok(())
}
