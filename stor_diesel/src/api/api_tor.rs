use crate::ModelTorrentsDiesel;
use crate::api::common::SQL_PLACEHOLDER_MAX;
use crate::connection::StorTransaction;
use crate::err::StorDieselResult;
use crate::models::diesel_wrappers::TorHashV1Diesel;
use crate::models::model_tor::ModelQbHost;
use crate::schema;
use diesel::{ExpressionMethods, HasQuery, QueryDsl, RunQueryDsl, TextExpressionMethods, dsl};
use itertools::Itertools;
use std::borrow::Borrow;
use xana_commons_rs::bencode_torrent_re::TorHashV1;

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
