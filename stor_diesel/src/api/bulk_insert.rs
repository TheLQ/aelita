use crate::api::common::check_insert_num_rows;
use crate::err::StorDieselErrorKind;
use crate::{StorDieselResult, StorTransaction, storapi_row_count};
use diesel::RunQueryDsl;
use diesel::connection::SimpleConnection;
use std::io::{BufWriter, Write};
use tempfile::NamedTempFile;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, CrashErrKind, LOCALE};

pub const DEFAULT_MEGA_CHUNK_SIZE: usize = 1_000_000;
const MEBI_BYTE: usize = 1024 * 1024;

pub const ROW_SEP: u8 = 0x1e;
pub const COL_SEP: u8 = 0x1f;

pub struct BulkyInsert<'c, 'tx, 't, 'k, 'v, Row, const COLS: usize> {
    pub conn: &'c mut StorTransaction<'tx>,
    pub table: &'t str,
    pub keys: [&'k str; COLS],
    pub values: &'v [Row],
    pub chunk_size: usize,
}

impl<'c, 'tx, 't, 'k, 'v, Row, const COLS: usize> BulkyInsert<'c, 'tx, 't, 'k, 'v, Row, COLS> {
    pub fn insert_probably_huge_data<Rowizer>(
        self,
        rowizer: Rowizer,
    ) -> StorDieselResult<Option<usize>>
    where
        Rowizer: Fn(RowizerContext<'v, '_, Row>) -> StorDieselResult<()>,
    {
        if self.values.len() < self.chunk_size * 2 {
            let rows = self.bulk_insert(rowizer);
            Some(rows).transpose()
        } else {
            self.bulk_load(rowizer)?;
            Ok(None)
        }
    }

    pub fn bulk_insert<'w, Rowizer>(self, rowizer: Rowizer) -> StorDieselResult<usize>
    where
        Rowizer: Fn(RowizerContext<'v, '_, Row>) -> StorDieselResult<()>,
    {
        let Self {
            conn,
            table,
            keys,
            values,
            chunk_size,
        } = self;
        let mut total_rows = 0;
        let prefix = format!("INSERT INTO `{table}` ({}) VALUES ", keys.join(","));
        let chunks = values.chunks(chunk_size);
        let chunks_len = chunks.len() - 1;
        for (chunk_i, chunk) in chunks.enumerate() {
            let mut query_raw = prefix.clone();
            for row in chunk {
                rowizer(RowizerContext {
                    before: "(",
                    middle: ",",
                    after: "),",
                    output: RowizerOut::String(&mut query_raw),
                    row,
                })?;
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

    fn bulk_load<Rowizer>(self, rowizer: Rowizer) -> StorDieselResult<()>
    where
        Rowizer: Fn(RowizerContext<'v, '_, Row>) -> StorDieselResult<()>,
    {
        let Self {
            conn,
            table,
            keys,
            values,
            chunk_size: _,
        } = self;
        let data_path = NamedTempFile::new().unwrap();
        let mut data_writer = BufWriter::new(data_path);

        let watch = BasicWatch::start();
        for row in values {
            rowizer(RowizerContext {
                before: "",
                // [COL_SEP]
                middle: "\u{1f}",
                // [ROW_SEP]
                after: "\u{1e}",
                output: RowizerOut::Temp(&mut data_writer),
                row,
            })?;
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
}

// pub struct RowizerContext<'v, 'w, W, Row, Output>
pub struct RowizerContext<'v, 'o, Row> {
    pub row: &'v Row,
    // pub output: &'w mut W,
    pub output: RowizerOut<'o>,
    //
    pub before: &'static str,
    pub middle: &'static str,
    pub after: &'static str,
}

pub enum RowizerOut<'i> {
    String(&'i mut String),
    Temp(&'i mut BufWriter<NamedTempFile>),
}

impl RowizerOut<'_> {
    pub fn add_single_row(self, row: std::fmt::Arguments) -> StorDieselResult<()> {
        match self {
            Self::String(s) => std::fmt::Write::write_fmt(s, row).map_err(|e| {
                StorDieselErrorKind::BadRowizerForBulkLoad
                    .build_message(format!("failed to format {e}"))
            }),
            Self::Temp(s) => s.write_fmt(row).map_err(|e| {
                StorDieselErrorKind::BadRowizerForBulkLoad
                    .build_message(format!("failed to format {e}"))
            }),
        }
    }
}
