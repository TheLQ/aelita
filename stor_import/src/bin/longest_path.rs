use aelita_commons::log_init;
use aelita_stor_import::err::StorImportResult;
use aelita_stor_import::{COMPRESSEDD_CACHE, CompressedPaths};
use std::path::PathBuf;
use std::process::ExitCode;
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{SimpleIoMap, pretty_main};

fn main() -> ExitCode {
    log_init();

    pretty_main(run)
}

fn run() -> StorImportResult<()> {
    let compressed_raw = std::fs::read(COMPRESSEDD_CACHE).map_io_err(COMPRESSEDD_CACHE)?;
    let compressed: CompressedPaths = postcard::from_bytes(&compressed_raw)?;

    let mut longest_path = PathBuf::new();
    let mut longest_path_len = 0;
    for path in compressed.iter_paths() {
        let cur_len = path.iter().map(|v| v.len()).sum();
        if cur_len > longest_path_len {
            longest_path_len = cur_len;
            longest_path = path;
        }
        // longest_path = longest_path.max();
    }

    info!(
        "found longest path {longest_path_len} chars at {}",
        longest_path.display()
    );
    for item in longest_path.iter() {
        info!("{} at {}", item.len(), item.to_str().unwrap());
    }

    Ok(())
}
