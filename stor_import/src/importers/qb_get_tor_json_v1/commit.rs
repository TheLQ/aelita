use crate::err::StorImportResult;
use crate::importers::qb_get_tor_json_v1::defs::ImportQbMetadata;
use crate::util::HashExtractor;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::api_tor::{
    storapi_tor_torrents_list_by_hash, storapi_tor_torrents_new,
    storapi_tor_torrents_update_status_batch,
};
use aelita_stor_diesel::id_types::{ModelJournalTypeName, ModelTorrentState};
use aelita_stor_diesel::model_journal::ModelJournalImmutable;
use aelita_stor_diesel::model_tor::ModelTorrents;
use xana_commons_rs::tracing_re::info;

pub fn storcommit_torrents(
    conn: &mut StorTransaction,
    row: ModelJournalImmutable,
) -> StorImportResult<()> {
    assert_eq!(row.journal_type, ModelJournalTypeName::QbGetTorJson1);

    let metadata: ImportQbMetadata = row.metadata.unwrap().deserialize_json()?;
    info!("meta {metadata:?}");

    // let meta = row.data;
    // let mut meta_str = String::from_utf8(meta).unwrap();
    // meta_str.truncate(3999);
    // info!("{}", meta_str);

    let local_tors: Vec<ModelTorrents> = row.data.deserialize_json()?;
    // let local_tors = local_tors_raw.as_tor_lookup_by_hash();

    let db_tors_raw =
        storapi_tor_torrents_list_by_hash(conn, local_tors.iter().map(|v| &v.infohash_v1))?;
    info!("existing {}", db_tors_raw.len());
    let db_tors = db_tors_raw.as_tor_lookup_by_hash();

    let mut rows_new = Vec::new();
    let mut rows_update = Vec::new();
    let mut count_existing_same = 0;
    let mut count_existing_changed = 0;
    for local_tor in local_tors {
        if let Some(model_tor) = db_tors.get(&local_tor.infohash_v1) {
            if model_tor.state.inner() == local_tor.state.inner() {
                // don't update anything
                count_existing_same += 1;
            } else {
                rows_update.push((local_tor.infohash_v1, local_tor.state.into_inner()));
                count_existing_changed += 1;
            }
        } else {
            rows_new.push(local_tor)
        }
    }
    info!(
        "torrents {} inserted {count_existing_same} same {count_existing_changed} changed",
        rows_new.len()
    );
    if !rows_update.is_empty() {
        storapi_tor_torrents_update_status_batch(conn, rows_update)?;
    }
    if !rows_new.is_empty() {
        storapi_tor_torrents_new(conn, rows_new)?;
    }

    Ok(())
}
