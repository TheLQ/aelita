use crate::api::common::{SQL_PLACEHOLDER_MAX, check_insert_num_rows};
use crate::{StorDieselResult, StorTransaction};
use diesel::RunQueryDsl;
use std::fmt::{Arguments, Display, Write};
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{CommaJoiner, LOCALE};

pub const DEFAULT_MEGA_CHUNK_SIZE: usize = 10_000_000;
const MEBI_BYTE: usize = 1024 * 1024;

pub fn bulk_insert<Row, const COLS: usize>(
    conn: &mut StorTransaction,
    table: &str,
    keys: [&str; COLS],
    values: &[Row],
    rowizer: impl Fn(&Row, &mut String),
    chunk_size: usize,
) -> StorDieselResult<usize> {
    let mut total_rows = 0;
    let prefix = format!("INSERT INTO `{table}` ({}) VALUES ", keys.join(","));
    let chunks = values.chunks(chunk_size);
    let chunks_len = chunks.len() - 1;
    for (chunk_i, chunk) in chunks.enumerate() {
        let mut query_raw = prefix.clone();
        for row in chunk {
            query_raw.push('(');
            rowizer(row, &mut query_raw);
            query_raw.push_str("),");
        }
        query_raw.pop();
        trace!(
            "Insert chunk {chunk_i} of {chunks_len} at {} MiB",
            (query_raw.len() / MEBI_BYTE).to_formatted_string(&LOCALE)
        );
        let rows = diesel::sql_query(&query_raw).execute(conn.inner());
        check_insert_num_rows(rows, chunk.len())?;
        total_rows += 1;
    }
    Ok(total_rows)
}
