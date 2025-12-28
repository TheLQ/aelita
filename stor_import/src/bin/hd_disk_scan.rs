use aelita_commons::log_init;
use aelita_stor_diesel::{
    PermaStore, StorTransaction, assert_packet_size_huge_enough, establish_connection_or_panic,
};
use aelita_stor_import::err::{StorImportError, StorImportResult};
use aelita_stor_import::storfetch_paths_from_disk;
use std::process::ExitCode;
use xana_commons_rs::pretty_main;

fn main() -> ExitCode {
    log_init();

    pretty_main(run)
}

fn run() -> StorImportResult<()> {
    let mut conn = establish_connection_or_panic(PermaStore::AelitaNull);
    assert_packet_size_huge_enough(&mut conn)?;

    StorTransaction::new_transaction("cli-import", &mut conn, |conn| {
        storfetch_paths_from_disk(conn, &["/dup18", "/che12"])?;
        Ok::<_, Box<StorImportError>>(())
    })?;

    Ok(())
}
