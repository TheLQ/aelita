use crate::err::StorImportResult;
use crate::importers::qb_get_tor_json_v1::defs::{ImportQbMetadata, ImportQbTorrent};
use crate::util::HashExtractor;
use aelita_stor_diesel::api_tor::{
    storapi_tor_torrents_get_by_hash, storapi_tor_torrents_new, storapi_tor_torrents_update_status,
    storapi_tor_torrents_update_status_batch,
};
use aelita_stor_diesel::id_types::ModelJournalTypeName;
use aelita_stor_diesel::model_journal::ModelJournalDataImmutable;
use aelita_stor_diesel::model_tor::NewModelTorrents;
use aelita_stor_diesel::{StorTransaction, with_quiet_sql_log_spam};
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
    info!("existing {}", db_tors_raw.len());
    let db_tors = db_tors_raw.as_tor_lookup_by_hash();

    let mut rows_new = Vec::new();
    let mut rows_update = Vec::new();
    let mut count_existing_same = 0;
    let mut count_existing_changed = 0;
    for (local_hash, local_tor) in local_tors {
        if let Some(model_tor) = db_tors.get(&local_hash) {
            if *model_tor.tor_status.inner() == local_tor.state {
                // don't update anything
                count_existing_same += 1;
            } else {
                rows_update.push((local_hash.clone(), local_tor.state));
                count_existing_changed += 1;
            }
        } else {
            rows_new.push(NewModelTorrents {
                journal_id: row.journal_id,
                torhash: local_hash.into(),
                qb_host_id: metadata.qb_host_id,
                tor_status: local_tor.state.into(),
            })
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
        storapi_tor_torrents_new(conn, &rows_new)?;
    }

    Ok(())
}
