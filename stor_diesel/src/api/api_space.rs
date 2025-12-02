use crate::api::common::{check_insert_num_rows, mysql_last_id};
use crate::connection::{StorConnection, assert_in_transaction};
use crate::err::StorDieselResult;
use crate::models::id_types::{ModelSpaceId, StorIdType};
use crate::models::model_space::{ModelSpace, ModelSpaceOwned, NewModelSpace};
use diesel::{HasQuery, QueryResult, RunQueryDsl};

pub fn storapi_space_new(
    conn: &mut StorConnection,
    space: NewModelSpace,
) -> StorDieselResult<ModelSpaceId> {
    assert_in_transaction();

    let rows = diesel::insert_into(crate::schema::space::table)
        .values(space)
        .execute(conn);
    check_insert_num_rows(rows, 1)?;
    Ok(ModelSpaceId::new(mysql_last_id(conn)?))
}

pub fn storapi_space_get(conn: &mut StorConnection) -> QueryResult<Vec<ModelSpace>> {
    assert_in_transaction();

    ModelSpace::query().load(conn)
}

pub fn storapi_space_owned_new(
    conn: &mut StorConnection,
    space: ModelSpaceOwned,
) -> StorDieselResult<()> {
    assert_in_transaction();

    let rows = diesel::insert_into(crate::schema::space_owned::table)
        .values(space)
        .execute(conn);
    check_insert_num_rows(rows, 1)
}

pub fn storapi_space_owned_get(conn: &mut StorConnection) -> QueryResult<Vec<ModelSpaceOwned>> {
    assert_in_transaction();

    ModelSpaceOwned::query().load(conn)
}
