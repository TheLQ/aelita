use crate::err::{StorImportErrorKind, StorImportResult};
use crate::importers::n_data_v1::path_backup::{ChannelOutSaved, read_input_cache};
use aelita_stor_diesel::ModelJournalTypeName;
use aelita_stor_diesel::NewModelJournalImmutable;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::err::StorDieselErrorKind::LoadInfileFailed;
use aelita_stor_diesel::path_const::PathConst;
use aelita_stor_diesel::storapi_journal_immutable_push_single;
use aelita_stor_diesel::{CompressedPaths, RawDieselBytes};
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::thread;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{error, info, trace, warn};
use xana_commons_rs::{
    BasicWatch, CrashErrKind, LOCALE, RecursiveStatResult, ResultXanaMap, ScanFileType,
    ScanFileTypeWithPath, ScanStat, SimpleIoMap, read_dirs_recursive_better,
    read_dirs_recursive_stat_better,
};

static ROOTS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let path = Path::new("local_data/ndata_roots.txt");
    let raw = std::fs::read_to_string(path).map_io_err(path).unwrap();
    raw.split('\n').map(|s| s.to_string()).collect()
});
pub const COMPRESSED_CACHE: PathConst = PathConst("compressed_paths.cache.json");
pub const INPUT_CACHE: PathConst = PathConst("compressed_paths.inputcache.json");

pub fn storfetch_paths_from_cache(conn: &mut StorTransaction) -> StorImportResult<()> {
    let compressed_bytes = std::fs::read(COMPRESSED_CACHE)
        .map_io_err(COMPRESSED_CACHE)
        .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;
    // insert_compressed_encoded(conn, RawDieselBytes(compressed_bytes))?;
    Ok(())
}

pub fn storfetch_paths_from_disk(
    conn: &mut StorTransaction,
    roots: &[impl AsRef<Path>],
) -> StorImportResult<()> {
    #[derive(Serialize, Deserialize)]
    struct ResWrapper {
        out: Vec<(ScanFileTypeWithPath, ScanStat)>,
    }

    const LOAD_FROM_DISK: bool = false;
    let scans = if LOAD_FROM_DISK || !INPUT_CACHE.exists() {
        let res = ResWrapper {
            out: scan_disk(roots),
        };
        // let raw = RawDieselBytes::serialize_postcard(&res)
        //     .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;
        // std::fs::write(INPUT_CACHE, raw.as_inner())
        //     .map_io_err(INPUT_CACHE)
        //     .unwrap();
        // info!(
        //     "Wrote {} bytes to cache {}",
        //     raw.0.len(),
        //     INPUT_CACHE.display()
        // );
        res.out
    } else {
        let watch = BasicWatch::start();
        let raw = std::fs::read(INPUT_CACHE).map_io_err(INPUT_CACHE).unwrap();
        trace!("Loaded {} from disk in {watch}", INPUT_CACHE.display());
        let res_cache = read_input_cache(&raw)?;
        let res = res_cache
            .into_iter()
            .map(|(disk, scan)| ((&disk).into(), scan))
            .collect();
        info!("Loaded from {} in {watch}", INPUT_CACHE.display());
        res
    };
    let raw_size: usize = scans
        .iter()
        .map(|v| {
            let path = v.0.path();
            path.as_os_str().len()
        })
        .sum();

    let compressed = CompressedPaths::from_scan(scans)?;
    let encoded = {
        let watch = BasicWatch::start();
        let post = RawDieselBytes::serialize_postcard(&compressed)
            .map_err(StorImportErrorKind::InvalidCompressedPaths.err_map())?;
        trace!("Postcard serialized in {watch}");

        let watch = BasicWatch::start();
        let real = zstd::encode_all(post.as_inner(), 0)
            .map_io_err("zstd-err")
            .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;
        trace!("ZFS serialized in {watch}");
        real
    };
    let encoded_size: usize = encoded.len();

    let saved_bytes = raw_size as isize - encoded_size as isize;
    let saved_percent = ((raw_size as f64 - encoded_size as f64) / raw_size as f64) * 100.0;
    info!(
        "encoded {} raw {} saved {} reduction {saved_percent:.1}%",
        encoded_size.to_formatted_string(&LOCALE),
        raw_size.to_formatted_string(&LOCALE),
        saved_bytes.to_formatted_string(&LOCALE)
    );

    std::fs::write(COMPRESSED_CACHE, &encoded)
        .map_io_err(COMPRESSED_CACHE)
        .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;
    info!("wrote to {}", COMPRESSED_CACHE.display());

    insert_compressed_encoded(conn, RawDieselBytes(encoded))?;
    Ok(())
}

fn scan_disk(roots: &[impl AsRef<Path>]) -> Vec<(ScanFileTypeWithPath, ScanStat)> {
    let total_watch = BasicWatch::start();
    let mut handles = Vec::new();
    let mut output = ChannelOutSaved::new(INPUT_CACHE.as_ref());

    let (output_send, output_recv) =
        std::sync::mpsc::channel::<Option<(ScanFileTypeWithPath, ScanStat)>>();

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
    output_send: std::sync::mpsc::Sender<Option<(ScanFileTypeWithPath, ScanStat)>>,
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

fn insert_compressed(
    conn: &mut StorTransaction,
    compressed: CompressedPaths,
) -> StorImportResult<()> {
    // let compressed_encoded = std::fs::read(COMPRESSEDD_CACHE).map_io_err(COMPRESSEDD_CACHE)?;
    // let data = RawDieselBytes(compressed_encoded);
    let data = RawDieselBytes::serialize_postcard(&compressed)
        .xana_err(StorImportErrorKind::InvalidCompressedPaths)?;
    insert_compressed_encoded(conn, data)
}

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
