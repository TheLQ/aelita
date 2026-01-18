use crate::err::{StorImportErrorKind, StorImportResult};
use crate::importers::n_data_v1::path_backup::{ChannelOutSaved, read_input_cache};
use aelita_stor_diesel::NewModelJournalImmutable;
use aelita_stor_diesel::RawDieselBytes;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::path_const::PathConst;
use aelita_stor_diesel::storapi_journal_immutable_push_single;
use aelita_stor_diesel::{ModelJournalTypeName, encode_compressed_paths};
use std::path::Path;
use std::sync::LazyLock;
use std::thread;
use xana_commons_rs::tracing_re::{debug, error, info};
use xana_commons_rs::{BasicWatch, CrashErrKind, ResultXanaMap, SimpleIoMap};
use xana_fs_indexer_rs::{CompressedPaths, RecursiveStatResult, read_dirs_recursive_stat_better};

pub const COMPRESSED_CACHE: PathConst = PathConst("compressed_paths.cache.json");
pub const SCAN_CACHE: PathConst = PathConst("compressed_paths.scancache.json");

pub fn storfetch_paths_from_cache(conn: &mut StorTransaction) -> StorImportResult<()> {
    let compressed_bytes = std::fs::read(COMPRESSED_CACHE)
        .map_io_err(COMPRESSED_CACHE)
        .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;
    insert_compressed_encoded(conn, RawDieselBytes(compressed_bytes))?;
    Ok(())
}

pub fn storfetch_paths_from_disk(
    conn: &mut StorTransaction,
    roots: &[impl AsRef<Path>],
) -> StorImportResult<()> {
    const LOAD_FROM_DISK: bool = false;
    let scans = if LOAD_FROM_DISK || !SCAN_CACHE.exists() {
        scan_disk(roots)
    } else {
        scan_disk_cached()?
    };

    let (_compressed, encoded) = stat_scan_to_compressed(scans)?;
    insert_compressed_encoded(conn, RawDieselBytes(encoded))?;
    Ok(())
}

fn scan_disk_cached() -> StorImportResult<Vec<RecursiveStatResult>> {
    info!("loading scan_disk from {}", SCAN_CACHE.display());
    let watch = BasicWatch::start();
    let res_cache = read_input_cache(SCAN_CACHE.as_ref())?;
    let res = res_cache
        .into_iter()
        .map(|(disk, scan)| ((&disk).into(), scan))
        .collect();
    info!("Loaded from {} in {watch}", SCAN_CACHE.display());
    Ok(res)
}

fn stat_scan_to_compressed(
    scans: Vec<RecursiveStatResult>,
) -> StorImportResult<(CompressedPaths, Vec<u8>)> {
    let raw_size: usize = scans
        .iter()
        .map(|v| {
            let path = v.0.path();
            path.as_os_str().len()
        })
        .sum();

    let watch = BasicWatch::start();
    let compressed = CompressedPaths::from_scan(scans, true)
        .map_err(StorImportErrorKind::InvalidCompressedPaths.xana_map())?;
    debug!("CompressedPath built in {watch}");

    let encoded = encode_compressed_paths(&compressed, Some(raw_size))?;

    std::fs::write(COMPRESSED_CACHE, &encoded)
        .map_io_err(COMPRESSED_CACHE)
        .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;
    info!("wrote to {}", COMPRESSED_CACHE.display());

    Ok((compressed, encoded))
}

fn scan_disk(roots: &[impl AsRef<Path>]) -> Vec<RecursiveStatResult> {
    let total_watch = BasicWatch::start();
    let mut handles = Vec::new();
    let mut output = ChannelOutSaved::new(SCAN_CACHE.as_ref());

    let (output_send, output_recv) = std::sync::mpsc::channel::<Option<RecursiveStatResult>>();

    for root in roots.iter() {
        let root = root.as_ref().to_path_buf();
        info!("scanning {}...", root.display());
        let output_send = output_send.clone();
        let handle = thread::Builder::new()
            .name(format!(
                "{:<17}",
                Path::new(&root).file_name().unwrap().display()
            ))
            .spawn(move || {
                let errors = scan_disk_root(&root, output_send.clone());
                output_send.send(None).unwrap();
                errors
            })
            .unwrap();
        handles.push(handle);
    }

    let mut total_complete = 0;
    loop {
        let next = output_recv.recv().unwrap();
        if let Some(next) = next {
            output.push(next)
        } else {
            total_complete += 1;
            if total_complete == roots.len() {
                break;
            }
        }
    }

    let mut total_errors = 0;
    for handle in handles {
        let errors = handle.join().unwrap();
        total_errors += errors;
    }

    let res = output.into_output();
    info!(
        "Scanned {} files with {total_errors} errors in {total_watch}",
        res.len(),
    );
    res
}

fn scan_disk_root(
    root: &Path,
    output_send: std::sync::mpsc::Sender<Option<RecursiveStatResult>>,
) -> usize {
    let watch = BasicWatch::start();
    let mut total_errors = 0;
    read_dirs_recursive_stat_better([root], |v| match v {
        Ok(v) => output_send.send(Some(v)).unwrap(),
        Err((path, e)) => {
            total_errors += 1;
            error!("failed {} because {}", path.display(), e)
        }
    });
    info!("Scanned {} in {watch}", root.display());
    total_errors
}

// fn insert_compressed(
//     conn: &mut StorTransaction,
//     compressed: CompressedPaths,
// ) -> StorImportResult<()> {
//     // let compressed_encoded = std::fs::read(COMPRESSEDD_CACHE).map_io_err(COMPRESSEDD_CACHE)?;
//     // let data = RawDieselBytes(compressed_encoded);
//     let data = RawDieselBytes::serialize_postcard(&compressed)
//         .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;
//     insert_compressed_encoded(conn, data)
// }

fn insert_compressed_encoded(
    conn: &mut StorTransaction,
    data: RawDieselBytes,
) -> StorImportResult<()> {
    let journal_id = storapi_journal_immutable_push_single(
        conn,
        NewModelJournalImmutable {
            journal_type: ModelJournalTypeName::NData1,
            data,
            metadata: None,
            cause_description: "disk-scanner".into(),
            cause_xrn: None,
        },
    )?;
    info!("inserted ndata journal_id {journal_id}");

    Ok(())
}
