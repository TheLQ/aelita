use crate::err::{StorDieselError, StorDieselResult};
use aelita_commons::err_utils::xbt;
use diesel::QueryResult;

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
