use crate::connection::{StorConnection, StorTransaction};
use crate::err::{StorDieselErrorKind, StorDieselResult};
use crate::storapi_variables_get;
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

pub fn assert_packet_size_huge_enough(conn: &mut StorConnection) -> StorDieselResult<()> {
    let max_packet_size = storapi_variables_get(conn, "max_allowed_packet")?;
    if max_packet_size < /*100 MiB*/100 * 1024 * 1024 {
        panic!(
            "too small packet size {max_packet_size} = {} MiB",
            max_packet_size / 1024 / 1024
        );
    } else {
        info!(
            "small packet size {max_packet_size} = {} MiB",
            max_packet_size / 1024 / 1024
        );
        Ok(())
    }
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

// pub struct ChunkyQuerySlice<'c, V, const CHUNK: usize>(&'c [V]);
//
// impl<'c, V, const CHUNK: usize> ChunkyQuerySlice<'c, V, CHUNK> {}
//
// impl<'c, V: 'c, const CHUNK: usize> IntoIterator for ChunkyQuerySlice<'c, V, CHUNK> {
//     type Item = V;
//     type IntoIter = std::slice::Iter<'c, V>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         self.0.iter()
//     }
// }

#[cfg(test)]
pub mod test {
    use crate::{PermaStore, StorDieselResult, StorTransaction, establish_connection};
    use aelita_commons::log_init;

    pub fn sql_test(
        inner: impl Fn(&mut StorTransaction) -> StorDieselResult<()>,
    ) -> StorDieselResult<()> {
        log_init();
        let conn = &mut establish_connection(PermaStore::AelitaNull).expect("bad conn");
        StorTransaction::new_transaction("test", conn, inner)?;
        Ok(())
    }
}
