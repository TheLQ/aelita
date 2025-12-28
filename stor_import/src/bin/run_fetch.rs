#![feature(iterator_try_collect)]

use aelita_commons::log_init;
use aelita_stor_diesel::{PermaStore, StorTransaction, establish_connection_or_panic};
use aelita_stor_diesel::{assert_packet_size_huge_enough, storapi_variables_get};
use aelita_stor_import::err::{StorImportError, StorImportResult};
use aelita_stor_import::storfetch_ndata;
use std::process::ExitCode;
use xana_commons_rs::pretty_main_async;
use xana_commons_rs::tracing_re::info;

fn main() -> ExitCode {
    log_init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("tokio-w")
        .build()
        .unwrap()
        .block_on(async { pretty_main_async(run).await })
}

async fn run() -> StorImportResult<()> {
    let mut conn = establish_connection_or_panic(PermaStore::AelitaNull);
    assert_packet_size_huge_enough(&mut conn);

    StorTransaction::new_transaction("cli-import", &mut conn, |conn| {
        storfetch_ndata(conn)?;
        // storfetch_torrents(conn)
        Ok::<_, StorImportError>(())
    })?;

    Ok(())
}
