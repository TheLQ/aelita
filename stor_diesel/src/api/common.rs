use crate::connection::StorConnection;
use crate::err::{StorDieselError, StorDieselResult};
use aelita_commons::err_utils::xbt;
use diesel::sql_types::{Integer, Text, Unsigned};
use diesel::{QueryResult, RunQueryDsl, dsl};
use xana_commons_rs::tracing_re::info;

pub fn check_insert_num_rows(query: QueryResult<usize>, expected: usize) -> StorDieselResult<()> {
    let result_size = query?;
    if result_size != expected {
        Err(StorDieselError::ResultLen {
            actual: result_size,
            expected,
            backtrace: xbt(),
        })
    } else {
        Ok(())
    }
}

pub fn with_counter<I, O>(counter: &mut u32, mapper: impl Fn(u32, I) -> O) -> impl FnMut(I) -> O {
    move |cur| {
        let res = mapper(*counter, cur);
        *counter += 1;
        res
    }
}

pub fn assert_test_database(conn: &mut StorConnection) -> QueryResult<()> {
    let db_name: String = diesel::select(diesel::dsl::sql::<Text>("DATABASE()")).first(conn)?;
    info!("database name: {}", db_name);
    assert_eq!(db_name, "aelita_null");
    Ok(())
}

pub fn mysql_last_id(conn: &mut StorConnection) -> QueryResult<u32> {
    diesel::select(dsl::sql::<Unsigned<Integer>>("LAST_INSERT_ID()")).first(conn)
}
