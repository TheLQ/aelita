use crate::api::common::{check_insert_num_rows, mysql_last_id};
use crate::{
    ModelSpaceId, ModelSpaceOwned, ModelSpaceXrn, NewModelSpaceName, StorDieselResult, StorIdType,
    StorTransaction, schema,
};
use diesel::prelude::*;
use diesel::{RunQueryDsl, dsl};

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
    spaces: impl IntoIterator<Item = (ModelSpaceOwned, ModelSpaceXrn)>,
) -> StorDieselResult<Vec<ModelSpaceId>> {
    let spaces = spaces
        .into_iter()
        .map(|(row, xrn)| {
            // let ModelSpaceOwned {
            //     journal_id,
            //     space_id,
            //     description,
            // } = row;

            // let type1_name: &str = X::atype().as_ref();
            // let child_type1 = schema::space_owned::child_type1.eq(AnyEnumToText::new(type1_name));
            // (row, child_type1)
            (row, xrn)
        })
        .collect::<Vec<_>>();

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
