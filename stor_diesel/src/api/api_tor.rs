use crate::api::common::{SQL_PLACEHOLDER_MAX, check_insert_num_rows, mysql_last_id};
use crate::connection::StorTransaction;
use crate::diesel_wrappers::TorHashV1Diesel;
use crate::err::StorDieselResult;
use crate::id_types::ModelTorrentState;
use crate::model_tor::{
    ModelSuperfast, ModelTorrentsDiesel, ModelTorrentsMeta, ModelTorrentsQBittorrent,
};
use crate::models::id_types::{ModelQbHostId, StorIdType};
use crate::models::model_tor::{ModelQbHost, NewModelQbHosts};
use crate::schema_temp::{SQL_FAST_TOR_CREATE, SQL_FAST_TOR_TRUNCATE};
use crate::{assert_test_database, schema};
use diesel::{
    ExpressionMethods, HasQuery, Insertable, QueryDsl, RunQueryDsl, TextExpressionMethods, dsl,
};
use itertools::Itertools;
use std::borrow::Borrow;
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

pub fn storapi_tor_host_list(conn: &mut StorTransaction) -> StorDieselResult<Vec<ModelQbHost>> {
    ModelQbHost::query()
        .get_results(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_tor_torrents_new(
    conn: &mut StorTransaction,
    meta: ModelTorrentsMeta,
    torrents: impl IntoIterator<Item = ModelTorrentsDiesel>,
) -> StorDieselResult<()> {
    let torrents = torrents.into_iter();
    for chunk in torrents
        .into_iter()
        .chunks(SQL_PLACEHOLDER_MAX / /*columns*/16)
        .into_iter()
    {
        let chunk = chunk.map(|v| (meta.clone(), v)).collect_vec();
        let expected_len = chunk.len();
        let rows = diesel::insert_into(schema::tor1_torrents::table)
            .values(chunk)
            .execute(conn.inner());
        check_insert_num_rows(rows, expected_len)?;
    }
    Ok(())
}

pub fn storapi_tor_torrents_list_starts_with(
    conn: &mut StorTransaction,
    starts_with: &str,
) -> StorDieselResult<Vec<ModelTorrentsDiesel>> {
    let mut query = ModelTorrentsDiesel::query().into_boxed();
    if !starts_with.is_empty() {
        query = query.filter(schema::tor1_torrents::name.like(format!("%{starts_with}%")));
    }
    query
        .limit(100)
        .get_results(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_tor_torrents_list_starts_with_count(
    conn: &mut StorTransaction,
    starts_with: &str,
) -> StorDieselResult<i64> {
    let mut query = schema::tor1_torrents::table
        .select(dsl::count_star())
        .into_boxed();
    if !starts_with.is_empty() {
        query = query.filter(schema::tor1_torrents::name.like(format!("%{starts_with}%")));
    }
    query.first::<i64>(conn.inner()).map_err(Into::into)
}

pub fn storapi_tor_torrents_list_by_hash(
    conn: &mut StorTransaction,
    info_hashes: impl IntoIterator<Item = impl Borrow<TorHashV1>>,
) -> StorDieselResult<Vec<ModelTorrentsDiesel>> {
    let mut res = Vec::new();
    for chunk in info_hashes
        .into_iter()
        .chunks(SQL_PLACEHOLDER_MAX)
        .into_iter()
    {
        let chunk: Vec<TorHashV1Diesel> = chunk.map(|v| v.borrow().into()).collect();
        let rows = ModelTorrentsDiesel::query()
            .filter(schema::tor1_torrents::infohash_v1.eq_any(chunk))
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
        .filter(schema::tor1_torrents::infohash_v2.eq(TorHashV1Diesel::from(hash)))
        .set(schema::tor1_torrents::state.eq(ModelTorrentState::from(state)))
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)
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
                tor_hash: TorHashV1Diesel::from(hash),
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

pub fn storapi_tor_reset(conn: &mut StorTransaction) -> StorDieselResult<()> {
    assert_test_database(conn)?;
    diesel::delete(schema::tor1_qb_host::table).execute(conn.inner())?;
    diesel::delete(schema::tor1_torrents::table).execute(conn.inner())?;
    Ok(())
}
