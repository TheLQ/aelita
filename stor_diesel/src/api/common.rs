use crate::connection::{StorConnection, StorTransaction};
use crate::err::{StorDieselErrorKind, StorDieselResult};
use diesel::sql_types::{Integer, Text, Unsigned};
use diesel::{QueryResult, QueryableByName, RunQueryDsl, dsl};
use xana_commons_rs::CrashErrKind;
use xana_commons_rs::tracing_re::info;

/// Avoid "Prepared statement contains too many placeholders"
pub const SQL_PLACEHOLDER_MAX: usize = 60_000;

pub fn check_insert_num_rows(query: QueryResult<usize>, expected: usize) -> StorDieselResult<()> {
    let result_size = query?;
    if result_size != expected {
        Err(StorDieselErrorKind::ResultLen
            .build_message(format!("actual {result_size} expected {expected}")))
    } else {
        Ok(())
    }
}

pub fn assert_test_database(conn: &mut StorTransaction) -> QueryResult<()> {
    let db_name: String = diesel::select(dsl::sql::<Text>("DATABASE()")).first(conn.inner())?;
    info!("database name: {}", db_name);
    assert_eq!(db_name, "aelita_null");
    Ok(())
}

pub fn mysql_last_id(conn: &mut StorConnection) -> QueryResult<u32> {
    diesel::select(dsl::sql::<Unsigned<Integer>>("LAST_INSERT_ID()")).first(conn)
}

/// todo doesn't work
pub fn show_create_table(
    conn: &mut StorConnection,
    table: impl Into<String>,
) -> QueryResult<String> {
    let table = table.into();
    let row = diesel::sql_query(&format!("SHOW CREATE TABLE `{table}`"))
        .get_result::<CreateResult>(conn)?;
    Ok(row.create_table)
}

#[derive(QueryableByName)]
struct CreateResult {
    #[diesel(sql_type = Text)]
    create_table: String,
}
