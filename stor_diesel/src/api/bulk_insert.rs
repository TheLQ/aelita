use crate::api::common::check_insert_num_rows;
use crate::{StorDieselResult, StorTransaction, storapi_row_count};
use diesel::RunQueryDsl;
use diesel::connection::SimpleConnection;
use std::io::BufWriter;
use tempfile::NamedTempFile;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, LOCALE};

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

pub const ROW_SEP: u8 = 0x1e;
pub const COL_SEP: u8 = 0x1f;
pub fn bulk_load<Row, const COLS: usize>(
    conn: &mut StorTransaction,
    table: &str,
    keys: [&str; COLS],
    values: &[Row],
    rowizer: impl Fn(&Row, &mut BufWriter<NamedTempFile>),
) -> StorDieselResult<()> {
    let data_path = NamedTempFile::new().unwrap();
    let mut data_writer = BufWriter::new(data_path);

    let watch = BasicWatch::start();
    for row in values {
        rowizer(row, &mut data_writer)
    }
    let data_path = data_writer.into_inner().unwrap();
    let path = data_path.path();
    info!(
        "Parsed {} rows in {watch} into {}",
        values.len(),
        path.display()
    );

    let watch = BasicWatch::start();
    let keys = keys.join(",");
    conn.inner().batch_execute(&format!(
        "LOAD DATA LOCAL INFILE '{}' \
        INTO TABLE `{table}` \
        FIELDS TERMINATED BY '{COL_SEP}' \
        LINES TERMINATED BY '{ROW_SEP}' \
        ({keys})",
        path.display()
    ))?;
    info!("LOAD DATA in {watch} count {}", storapi_row_count(conn)?);

    Ok(())
}
