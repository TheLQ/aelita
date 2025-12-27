use crate::api::assert_test_database;
use crate::connection::StorTransaction;
use crate::err::StorDieselResult;
use crate::models::id_types::ModelSpaceId;
use crate::models::model_space::{ModelSpaceName, ModelSpaceOwned};
use crate::schema;
use diesel::prelude::*;
use diesel::{HasQuery, QueryDsl, RunQueryDsl};
use std::ops::Range;
use xana_commons_rs::tracing_re::info;

pub fn storapi_space_get(
    conn: &mut StorTransaction,
    space_id: ModelSpaceId,
) -> StorDieselResult<ModelSpaceName> {
    ModelSpaceName::query()
        .filter(schema::space_names::space_id.eq(space_id))
        .first(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_space_list(conn: &mut StorTransaction) -> StorDieselResult<Vec<ModelSpaceName>> {
    ModelSpaceName::query()
        .load(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_space_list_filtered(
    conn: &mut StorTransaction,
    range: Range<u32>,
) -> StorDieselResult<Vec<ModelSpaceName>> {
    ModelSpaceName::query()
        .filter(schema::space_names::space_id.between(range.start, range.end))
        .load(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_space_owned_list(
    conn: &mut StorTransaction,
) -> StorDieselResult<Vec<ModelSpaceOwned>> {
    ModelSpaceOwned::query()
        .load(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_reset_space(conn: &mut StorTransaction) -> StorDieselResult<()> {
    assert_test_database(conn)?;
    let space_owned = diesel::delete(schema::space_owned::table).execute(conn.inner())?;
    let space_names = diesel::delete(schema::space_names::table).execute(conn.inner())?;
    info!("Reset {space_names} names {space_owned} owned rows");
    Ok(())
}
