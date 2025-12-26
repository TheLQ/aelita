use crate::err::{StorImportErrorKind, StorImportResult};
use crate::importers::qb_get_tor_json_v1::defs::ImportQbMetadata;
use aelita_stor_diesel::ModelJournalImmutable;
use aelita_stor_diesel::ModelJournalTypeName;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::storapi_tor_torrents_list_by_hash;
use aelita_stor_diesel::{ModelTorrentsDiesel, ModelTorrentsMeta, ModelTorrentsQBittorrent};
use aelita_stor_diesel::{storapi_tor_torrents_push, storapi_tor_torrents_update_status_batch};
use xana_commons_rs::bencode_torrent_re::HashExtractorAs;
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{CrashErrKind, ResultXanaMap};

pub fn storcommit_torrents(
    conn: &mut StorTransaction,
    row: ModelJournalImmutable,
) -> StorImportResult<()> {
    assert_eq!(row.journal_type, ModelJournalTypeName::QbGetTorJson1);

    let metadata: ImportQbMetadata = row
        .metadata
        .unwrap()
        .deserialize_json()
        .map_err(|(msg, e)| StorImportErrorKind::InvalidQbMetadata.build_err_message(e, msg))?;
    info!("meta {metadata:?}");

    // let meta = row.data;
    // let mut meta_str = String::from_utf8(meta).unwrap();
    // meta_str.truncate(3999);
    // info!("{}", meta_str);

    let local_tors: Vec<ModelTorrentsQBittorrent> = row
        .data
        .deserialize_json()
        .map_err(|(msg, e)| StorImportErrorKind::InvalidQbTorrents.build_err_message(e, msg))?;
    // let local_tors = local_tors_raw.as_tor_lookup_by_hash();

    let db_tors_raw =
        storapi_tor_torrents_list_by_hash(conn, local_tors.iter().map(|v| &v.infohash_v1))?;
    info!("existing {}", db_tors_raw.len());
    let db_tors = db_tors_raw.as_tor_lookup_by_hash();

    let mut rows_new: Vec<ModelTorrentsDiesel> = Vec::new();
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
            rows_new.push(local_tor.try_into()?)
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
        let meta = ModelTorrentsMeta {
            journal_id: row.journal_id,
            qb_host_id: metadata.qb_host_id,
        };
        storapi_tor_torrents_push(conn, meta, rows_new)?;
    }

    Ok(())
}
