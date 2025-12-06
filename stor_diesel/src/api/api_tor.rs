use crate::api::common::{check_insert_num_rows, mysql_last_id};
use crate::connection::StorTransaction;
use crate::err::StorDieselResult;
use crate::models::id_types::{ModelQbHostId, StorIdType};
use crate::models::model_tor::{ModelQbHost, ModelTorrents, NewModelQbHosts};
use crate::schema;
use diesel::{HasQuery, Insertable, RunQueryDsl};

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
    torrents: &[ModelTorrents],
) -> StorDieselResult<()> {
    let rows = diesel::insert_into(schema::tor1_torrents::table)
        .values(torrents)
        .execute(conn.inner());
    check_insert_num_rows(rows, torrents.len())
}

pub fn storapi_tor_torrents_get(
    conn: &mut StorTransaction,
) -> StorDieselResult<Vec<ModelTorrents>> {
    ModelTorrents::query()
        .get_results(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_tor_reset(conn: &mut StorTransaction) -> StorDieselResult<()> {
    diesel::delete(schema::tor1_qb_host::table).execute(conn.inner())?;
    diesel::delete(schema::tor1_torrents::table).execute(conn.inner())?;
    Ok(())
}
