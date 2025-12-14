use crate::StorDieselResult;
use crate::connection::StorConnection;
use diesel::sql_types::{Integer, Text};
use diesel::{RunQueryDsl, dsl};

// todo: diesel::sql_query expects everything to be untyped?
// pub fn storapi_variables_get(
//     conn: &mut StorConnection,
//     name: impl AsRef<str>,
// ) -> StorDieselResult<i32> {
//     // LIKE '{filter_like}'
//     let model: ModelVariable = diesel::sql_query(format!("SHOW VARIABLES ")).get_result(conn)?;
//     Ok(model.value)
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

pub fn storapi_variables_get_str(
    conn: &mut StorConnection,
    name: impl AsRef<str>,
) -> StorDieselResult<String> {
    let name = name.as_ref();
    diesel::select(dsl::sql::<Text>(&format!("@@GLOBAL.{name}")))
        .get_result(conn)
        .map_err(Into::into)
}
