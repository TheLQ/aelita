use crate::api::common::{check_insert_num_rows, mysql_last_id};
use crate::{
    ModelSpaceId, ModelSpaceOwned, NewModelSpaceName, StorDieselResult, StorIdType,
    StorTransaction, XrnAsOwnedTable, schema,
};
use diesel::RunQueryDsl;

pub fn storapi_space_new(
    conn: &mut StorTransaction,
    space: NewModelSpaceName,
) -> StorDieselResult<ModelSpaceId> {
    let rows = diesel::insert_into(schema::space_names::table)
        .values(space)
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)?;
    Ok(ModelSpaceId::new(mysql_last_id(conn.inner())?))
}

pub fn storapi_space_owned_new(
    conn: &mut StorTransaction,
    owned: ModelSpaceOwned,
    xrn: XrnAsOwnedTable,
) -> StorDieselResult<u32> {
    let rows = diesel::insert_into(schema::space_owned::table)
        .values((owned, xrn))
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)?;

    Ok(mysql_last_id(conn.inner())?)
}
