use crate::StorDieselResult;
use crate::connection::StorConnection;
use diesel::sql_types::Integer;
use diesel::{RunQueryDsl, dsl};

// todo: diesel::sql_query expects everything to be untyped?
// pub fn storapi_variables_get(
//     conn: &mut StorConnection,
//     filter_like: impl Display,
// ) -> StorDieselResult<ModelVariable> {
//     // LIKE '{filter_like}'
//     diesel::sql_query(format!("SHOW VARIABLES "))
//         .get_result(conn)
//         .map_err(Into::into)
// }

pub fn storapi_variables_get(
    conn: &mut StorConnection,
    name: impl AsRef<str>,
) -> StorDieselResult<i32> {
    let name = name.as_ref();
    diesel::select(dsl::sql::<Integer>(&format!("@@GLOBAL.{name}")))
        .get_result(conn)
        .map_err(Into::into)
}
