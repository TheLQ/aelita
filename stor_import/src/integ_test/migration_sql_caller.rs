use crate::err::{StorImportErrorKind, StorImportResult};
use aelita_stor_diesel::StorTransaction;
use std::path::Path;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{CrashErrKind, ResultXanaMap, io_op};

pub enum MigrationModel {
    Journal,
    Hd,
}

impl MigrationModel {
    fn load_file(&self) -> StorImportResult<String> {
        let path = match self {
            Self::Journal => Path::new("../stor_diesel/migrations/1_init_journal/up.sql"),
            Self::Hd => Path::new("../stor_diesel/migrations/4_init_hd/up.sql"),
        };
        trace!("Reading migration file {}", path.display());
        let content = io_op(path, |v| std::fs::read_to_string(v))
            .xana_err(StorImportErrorKind::DieselFailed)?;
        Ok(content)
    }

    pub fn create_table(&self, conn: &mut StorTransaction, table: &str) -> StorImportResult<()> {
        trace!("creating table {table}");
        let content = self.load_file()?;
        let Some(start) = content.find(&format!("CREATE TABLE IF NOT EXISTS `{table}`")) else {
            return Err(StorImportErrorKind::MigrationMissingCreate.build());
        };
        let remain = &content[start..];
        let needle = ");";
        let Some(end) = remain.find(needle) else {
            return Err(StorImportErrorKind::MigrationMissingEnd.build());
        };
        let command = &remain[..(end + needle.len())];

        info!("Create table {table}");
        conn.raw_sql_execute(command)
            .xana_err(StorImportErrorKind::DieselFailed)?;

        Ok(())
    }
}
