use crate::api::common::{SQL_PLACEHOLDER_MAX, check_insert_num_rows, mysql_last_id};
use crate::connection::StorTransaction;
use crate::err::StorDieselResult;
use crate::id_types::ModelTorrentState;
use crate::model_tor::NewModelTorrents;
use crate::models::id_types::{ModelQbHostId, StorIdType};
use crate::models::model_tor::{ModelQbHost, ModelTorrents, NewModelQbHosts};
use crate::schema;
use crate::util_types::TorHashV1Diesel;
use diesel::{ExpressionMethods, HasQuery, Insertable, QueryDsl, RunQueryDsl};
use xana_commons_rs::bencode_torrent_re::TorHashV1;
use xana_commons_rs::qbittorrent_re::TorrentState;

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

pub fn storapi_tor_host_get(conn: &mut StorTransaction) -> StorDieselResult<Vec<ModelQbHost>> {
    ModelQbHost::query()
        .get_results(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_tor_torrents_new(
    conn: &mut StorTransaction,
    torrents: &[NewModelTorrents],
) -> StorDieselResult<()> {
    for chunk in torrents.chunks(SQL_PLACEHOLDER_MAX / /*columns*/4) {
        let rows = diesel::insert_into(schema::tor1_torrents::table)
            .values(chunk)
            .execute(conn.inner());
        check_insert_num_rows(rows, chunk.len())?;
    }
    Ok(())
}

pub fn storapi_tor_torrents_get_by_hash(
    conn: &mut StorTransaction,
    info_hashes: &[TorHashV1Diesel],
) -> StorDieselResult<Vec<ModelTorrents>> {
    let mut res = Vec::new();
    for chunk in info_hashes.chunks(SQL_PLACEHOLDER_MAX) {
        let rows = ModelTorrents::query()
            .filter(schema::tor1_torrents::torhash.eq_any(chunk))
            .get_results(conn.inner())?;
        res.extend(rows);
    }
    Ok(res)
}

pub fn storapi_tor_torrents_update_status(
    conn: &mut StorTransaction,
    hash: &TorHashV1,
    state: &TorrentState,
) -> StorDieselResult<()> {
    let rows = diesel::update(schema::tor1_torrents::table)
        .filter(schema::tor1_torrents::torhash.eq(TorHashV1Diesel::from(hash)))
        .set(schema::tor1_torrents::tor_status.eq(ModelTorrentState::from(state)))
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)
}

pub fn storapi_tor_reset(conn: &mut StorTransaction) -> StorDieselResult<()> {
    diesel::delete(schema::tor1_qb_host::table).execute(conn.inner())?;
    diesel::delete(schema::tor1_torrents::table).execute(conn.inner())?;
    Ok(())
}
