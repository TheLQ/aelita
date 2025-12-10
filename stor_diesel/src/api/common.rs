use crate::connection::{StorConnection, StorTransaction};
use crate::err::{StorDieselError, StorDieselResult};
use diesel::sql_types::{Integer, Text, Unsigned};
use diesel::{QueryResult, RunQueryDsl, dsl};
use std::backtrace::Backtrace;
use xana_commons_rs::tracing_re::info;

/// Avoid "Prepared statement contains too many placeholders"
pub const SQL_PLACEHOLDER_MAX: usize = 60_000;

pub fn check_insert_num_rows(query: QueryResult<usize>, expected: usize) -> StorDieselResult<()> {
    let result_size = query?;
    if result_size != expected {
        Err(StorDieselError::ResultLen {
            actual: result_size,
            expected,
            backtrace: Backtrace::capture(),
        })
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
