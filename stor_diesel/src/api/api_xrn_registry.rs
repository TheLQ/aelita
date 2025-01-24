use crate::api::common::{StorConnection, check_insert_num_rows};
use crate::err::StorDieselResult;
use crate::models::{NewXrnExtraction, XrnExtraction};
use crate::schema::xrn_registry;
use diesel::dsl::insert_into;
use diesel::prelude::*;

pub fn storapi_xrns_list(connection: &mut StorConnection) -> StorDieselResult<Vec<XrnExtraction>> {
    xrn_registry::table
        .select(XrnExtraction::as_select())
        .load(connection)
        .map_err(Into::into)
}

pub fn storapi_xrns_push(
    connection: &mut StorConnection,
    new: Vec<NewXrnExtraction>,
) -> StorDieselResult<()> {
    check_insert_num_rows(
        new.len(),
        insert_into(xrn_registry::table)
            .values(new)
            .execute(connection),
    )
}
