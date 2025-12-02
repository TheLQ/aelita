use crate::api::common::{check_insert_num_rows, mysql_last_id};
use crate::connection::{StorConnection, StorTransaction};
use crate::err::StorDieselResult;
use crate::models::id_types::{ModelSpaceId, StorIdType};
use crate::models::model_space::{ModelSpaceNames, ModelSpaceOwned, NewModelSpaceNames};
use crate::schema;
use diesel::prelude::*;
use diesel::{HasQuery, QueryDsl, RunQueryDsl, dsl};
use std::ops::{DerefMut, Range};
use xana_commons_rs::tracing_re::info;

pub fn storapi_space_new(
    conn: &mut StorTransaction,
    space: NewModelSpaceNames,
) -> StorDieselResult<ModelSpaceId> {
    let rows = diesel::insert_into(schema::space_names::table)
        .values(space)
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)?;
    Ok(ModelSpaceId::new(mysql_last_id(conn.inner())?))
}

pub fn storapi_space_get(conn: &mut StorTransaction) -> StorDieselResult<Vec<ModelSpaceNames>> {
    ModelSpaceNames::query()
        .load(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_space_get_filtered(
    conn: &mut StorTransaction,
    range: Range<u32>,
) -> StorDieselResult<Vec<ModelSpaceNames>> {
    ModelSpaceNames::query()
        .filter(schema::space_names::space_id.between(range.start, range.end))
        .load(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_space_owned_new(
    conn: &mut StorTransaction,
    spaces: &[ModelSpaceOwned],
) -> StorDieselResult<Vec<ModelSpaceId>> {
    let max_id: Option<ModelSpaceId> = schema::space_names::table
        .select(dsl::max(schema::space_names::space_id))
        .get_result(conn.inner())?;
    let rows = diesel::insert_into(schema::space_owned::table)
        .values(spaces)
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)?;

    if let Some(max_id) = max_id {
        Ok(schema::space_names::table
            .select(schema::space_names::space_id)
            .filter(schema::space_names::space_id.gt(max_id))
            .get_results(conn.inner())?)
    } else {
        Ok(schema::space_names::table
            .select(schema::space_names::space_id)
            .get_results(conn.inner())?)
    }
}

pub fn storapi_space_owned_get(
    conn: &mut StorTransaction,
) -> StorDieselResult<Vec<ModelSpaceOwned>> {
    ModelSpaceOwned::query()
        .load(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_reset_space(conn: &mut StorTransaction) -> StorDieselResult<()> {
    let space_owned = diesel::delete(schema::space_owned::table).execute(conn.inner())?;
    let space_names = diesel::delete(schema::space_names::table).execute(conn.inner())?;
    info!("Reset {space_names} names {space_owned} owned rows");
    Ok(())
}
