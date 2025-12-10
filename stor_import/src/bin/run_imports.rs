#![feature(iterator_try_collect)]

use aelita_commons::log_init;
use aelita_stor_diesel::api_variables::storapi_variables_get;
use aelita_stor_diesel::{PermaStore, StorTransaction, establish_connection_or_panic};
use aelita_stor_import::err::StorImportResult;
use aelita_stor_import::storfetch_torrents;
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

    let max_packet_size = storapi_variables_get(&mut conn, "max_allowed_packet")?;
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
    }

    StorTransaction::new_transaction("cli-import", &mut conn, |conn| storfetch_torrents(conn))?;

    Ok(())
}
