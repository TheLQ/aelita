use aelita_commons::log_init;
use aelita_stor_diesel::compressed_paths::CompressedPaths;
use aelita_stor_diesel::diesel_wrappers::RawDieselBytes;
use aelita_stor_import::COMPRESSEDD_CACHE;
use aelita_stor_import::err::StorImportResult;
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
    let compressed: CompressedPaths = RawDieselBytes(compressed_raw).deserialize_postcard()?;

    let mut longest_pieces = 0;
    let mut longest_pieces_at = 0;
    let mut longest_path = PathBuf::new();
    let mut longest_path_len = 0;
    let mut total_paths = 0;
    for path in compressed.iter_paths() {
        total_paths += 1;
        let cur_pieces = path.iter().count();
        if cur_pieces > longest_pieces {
            longest_pieces = cur_pieces;
            longest_pieces_at = 0;
        }
        if cur_pieces == longest_pieces {
            longest_pieces_at += 1;
        }

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
    info!("longest pieces {longest_pieces} count {longest_pieces_at}");
    info!("total paths {total_paths}");
    Ok(())
}
