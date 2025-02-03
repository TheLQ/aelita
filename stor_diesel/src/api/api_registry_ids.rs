use crate::api::common::{StorConnection, check_insert_num_rows};
use crate::err::StorDieselResult;
use crate::models::{ModelRegistryId, ModelRegistryIdSql};
use crate::schema::registry_ids;
use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;

pub fn storapi_registry_ids_list(
    conn: &mut StorConnection,
) -> StorDieselResult<Vec<ModelRegistryId>> {
    let res: Vec<ModelRegistryIdSql> = registry_ids::table
        .select(ModelRegistryIdSql::as_select())
        .load(conn)?;
    res.into_iter().map(|v| v.try_into()).try_collect()
}

pub fn storapi_registry_ids_push(
    conn: &mut StorConnection,
    new: Vec<ModelRegistryId>,
) -> StorDieselResult<()> {
    let new: Vec<ModelRegistryIdSql> = new.into_iter().map(TryFrom::try_from).try_collect()?;
    check_insert_num_rows(
        insert_into(registry_ids::table).values(&new).execute(conn),
        new.len(),
    )
}

pub fn storapi_registry_ids_reset(conn: &mut StorConnection) -> StorDieselResult<usize> {
    Ok(delete(registry_ids::table).execute(conn)?)
}
