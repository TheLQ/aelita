#![feature(iterator_try_collect)]

use aelita_stor_diesel::{PermaStore, StorTransaction, establish_connection_or_panic};
use aelita_stor_import::common::log_init_trace;
use aelita_stor_import::err::StorImportResult;
use aelita_stor_import::storfetch_journal_torrents;
use std::process::ExitCode;
use xana_commons_rs::pretty_main_async;

fn main() -> ExitCode {
    log_init_trace();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("tokio-w")
        .build()
        .unwrap()
        .block_on(async { pretty_main_async(run).await })
}

async fn run() -> StorImportResult<()> {
    let mut conn = establish_connection_or_panic(PermaStore::AelitaNull);

    StorTransaction::new_transaction("cli-import", &mut conn, |conn| {
        storfetch_journal_torrents(conn)
    })?;

    Ok(())
}
