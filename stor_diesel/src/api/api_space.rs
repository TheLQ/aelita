use crate::api::common::{check_insert_num_rows, mysql_last_id};
use crate::connection::{StorConnection, assert_in_transaction};
use crate::err::StorDieselResult;
use crate::models::id_types::{ModelSpaceId, StorIdType};
use crate::models::model_space::{ModelSpaceNames, ModelSpaceOwned, NewModelSpaceNames};
use diesel::prelude::*;
use diesel::{HasQuery, QueryDsl, QueryResult, RunQueryDsl, dsl};

pub fn storapi_space_new(
    conn: &mut StorConnection,
    space: NewModelSpaceNames,
) -> StorDieselResult<ModelSpaceId> {
    assert_in_transaction();

    let rows = diesel::insert_into(crate::schema::space_names::table)
        .values(space)
        .execute(conn);
    check_insert_num_rows(rows, 1)?;
    Ok(ModelSpaceId::new(mysql_last_id(conn)?))
}

pub fn storapi_space_get(conn: &mut StorConnection) -> QueryResult<Vec<ModelSpaceNames>> {
    assert_in_transaction();

    ModelSpaceNames::query().load(conn)
}

pub fn storapi_space_owned_new(
    conn: &mut StorConnection,
    spaces: &[ModelSpaceOwned],
) -> StorDieselResult<Vec<ModelSpaceId>> {
    let max_id: Option<ModelSpaceId> = crate::schema::space_names::table
        .select(dsl::max(crate::schema::space_names::space_id))
        .get_result(conn)?;
    let rows = diesel::insert_into(crate::schema::space_owned::table)
        .values(spaces)
        .execute(conn);
    check_insert_num_rows(rows, 1)?;

    if let Some(max_id) = max_id {
        Ok(crate::schema::space_names::table
            .select(crate::schema::space_names::space_id)
            .filter(crate::schema::space_names::space_id.gt(max_id))
            .get_results(conn)?)
    } else {
        Ok(crate::schema::space_names::table
            .select(crate::schema::space_names::space_id)
            .get_results(conn)?)
    }
}

pub fn storapi_space_owned_get(conn: &mut StorConnection) -> QueryResult<Vec<ModelSpaceOwned>> {
    assert_in_transaction();

    ModelSpaceOwned::query().load(conn)
}
