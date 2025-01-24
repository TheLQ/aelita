use crate::err::{StorDieselError, StorDieselResult};
use aelita_commons::err_utils::xbt;
use diesel::{MysqlConnection, QueryResult};

/// todo: How to do this with "impl Trait"?
pub type StorConnection = MysqlConnection;

pub fn check_insert_num_rows(
    expected_len: usize,
    query: QueryResult<usize>,
) -> StorDieselResult<()> {
    let result_size = query?;
    if result_size != expected_len {
        Err(StorDieselError::ResultLen(result_size, expected_len, xbt()))
    } else {
        Ok(())
    }
}
