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
use xana_commons_rs::tracing_re::trace;

pub fn storapi_tor_host_list(conn: &mut StorTransaction) -> StorDieselResult<Vec<ModelQbHost>> {
    ModelQbHost::query()
        .get_results(conn.inner())
        .map_err(Into::into)
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
