use crate::api::assert_test_database;
use crate::api::common::{SQL_PLACEHOLDER_MAX, check_insert_num_rows, mysql_last_id};
use crate::models::diesel_wrappers::TorHashV1Diesel;
use crate::schema_temp::{SQL_FAST_TOR_CREATE, SQL_FAST_TOR_TRUNCATE};
use crate::{ModelQbHostId, ModelTorrentState, StorIdType};
use crate::{ModelSuperfast, ModelTorrentsDiesel, ModelTorrentsMeta, NewModelQbHosts};
use crate::{StorDieselResult, StorTransaction, schema};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use itertools::Itertools;
use xana_commons_rs::bencode_torrent_re::TorHashV1;
use xana_commons_rs::qbittorrent_re::TorrentState;
use xana_commons_rs::tracing_re::trace;

pub fn storapi_tor_host_new(
    conn: &mut StorTransaction,
    host: NewModelQbHosts,
) -> StorDieselResult<ModelQbHostId> {
    let rows = host
        .insert_into(schema::tor1_qb_host::table)
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)?;
    match mysql_last_id(conn.inner()) {
        Ok(id) => Ok(ModelQbHostId::new(id)),
        Err(e) => Err(e.into()),
    }
}

pub fn storapi_tor_torrents_push(
    conn: &mut StorTransaction,
    meta: ModelTorrentsMeta,
    torrents: Vec<ModelTorrentsDiesel>,
) -> StorDieselResult<()> {
    const CHUNK_SIZE: usize = SQL_PLACEHOLDER_MAX / /*columns*/16;
    let total_chunks = torrents.as_chunks::<CHUNK_SIZE>().0.len();
    let chunks = torrents.into_iter().chunks(CHUNK_SIZE);
    for (i, chunk) in chunks.into_iter().enumerate() {
        trace!("Insert chunk {i} of {total_chunks}");
        let chunk = chunk.map(|v| (meta.clone(), v)).collect_vec();
        let expected_len = chunk.len();
        let rows = diesel::insert_into(schema::tor1_torrents::table)
            .values(chunk)
            .execute(conn.inner());
        check_insert_num_rows(rows, expected_len)?;
    }
    Ok(())
}

/// 10,000s of UPDATE queries is glacially slow vs UPDATE...INNER JOIN
pub fn storapi_tor_torrents_update_status_batch(
    conn: &mut StorTransaction,
    updates: Vec<(TorHashV1, TorrentState)>,
) -> StorDieselResult<()> {
    use crate::schema_temp;

    diesel::sql_query(SQL_FAST_TOR_CREATE).execute(conn.inner())?;
    diesel::sql_query(SQL_FAST_TOR_TRUNCATE).execute(conn.inner())?;

    for chunk in updates.chunks(SQL_PLACEHOLDER_MAX / /*columns*/2) {
        let values = chunk
            .iter()
            .map(|(hash, state)| ModelSuperfast {
                tor_hash: hash.clone(),
                tor_state: state.to_string(),
            })
            .collect::<Vec<_>>();

        let rows = diesel::insert_into(schema_temp::fast_tor_update::table)
            .values(values)
            .execute(conn.inner());
        check_insert_num_rows(rows, chunk.len())?;
    }

    // joins in update are unsupported https://github.com/diesel-rs/diesel/issues/1478
    // diesel::update(schema::tor1_torrents::table.inner_join(schema_temp::fast_tor_update::table))
    //     .set(schema::tor1_torrents::tor_status.eq(schema_temp::fast_tor_update::tor_state))
    //     .execute(conn.inner());
    let rows = diesel::sql_query(
        "UPDATE `tor1_torrents` \
         INNER JOIN `fast_tor_update` \
         ON `tor1_torrents`.`infohash_v1` = `fast_tor_update`.`tor_hash` \
         SET `tor1_torrents`.`state` = `fast_tor_update`.`tor_state`",
    )
    .execute(conn.inner());
    check_insert_num_rows(rows, updates.len())?;

    Ok(())
}

pub fn storapi_tor_torrents_update_status(
    conn: &mut StorTransaction,
    hash: &TorHashV1,
    state: &TorrentState,
) -> StorDieselResult<()> {
    let rows = diesel::update(schema::tor1_torrents::table)
        .filter(schema::tor1_torrents::infohash_v2.eq(TorHashV1Diesel::from(hash)))
        .set(schema::tor1_torrents::state.eq(ModelTorrentState::from(state)))
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)
}

pub fn storapi_tor_reset(conn: &mut StorTransaction) -> StorDieselResult<()> {
    assert_test_database(conn)?;
    diesel::delete(schema::tor1_qb_host::table).execute(conn.inner())?;
    diesel::delete(schema::tor1_torrents::table).execute(conn.inner())?;
    Ok(())
}
