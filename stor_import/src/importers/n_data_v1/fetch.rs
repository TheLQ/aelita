use crate::err::{StorImportErrorKind, StorImportResult};
use aelita_stor_diesel::NewModelJournalImmutable;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::path_const::PathConst;
use aelita_stor_diesel::storapi_journal_immutable_push_single;
use aelita_stor_diesel::{CompressedPathNested, ModelJournalTypeName};
use aelita_stor_diesel::{CompressedPaths, RawDieselBytes};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::thread;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{error, info, trace, warn};
use xana_commons_rs::{
    BasicWatch, CrashErrKind, LOCALE, ResultXanaMap, ScanFileType, ScanFileTypeWithPath,
    SimpleIoMap, read_dirs_recursive_better,
};

static ROOTS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let path = Path::new("local_data/ndata_roots.txt");
    let raw = std::fs::read_to_string(path).map_io_err(path).unwrap();
    raw.split('\n').map(|s| s.to_string()).collect()
});
pub const COMPRESSED_CACHE: PathConst = PathConst("compressed_paths.cache.json");

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
    let scans = scan_disk(roots);
    let raw_size: usize = scans
        .iter()
        .map(|v| {
            let path = v.path();
            path.as_os_str().len()
        })
        .sum();

    let compressed = CompressedPathNested::from_scan(scans);
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

    insert_compressed_encoded(conn, RawDieselBytes(encoded))
    // todo!()
}

fn scan_disk(roots: &[impl AsRef<Path>]) -> Vec<ScanFileTypeWithPath> {
    let total_watch = BasicWatch::start();
    let mut handles = Vec::new();
    for root in roots.iter() {
        let root = root.as_ref().to_path_buf();
        info!("scanning {}...", root.display());
        let handle = thread::Builder::new()
            .name(format!(
                "{:<17}",
                Path::new(&root).file_name().unwrap().display()
            ))
            .spawn(move || {
                let watch = BasicWatch::start();
                let res = read_dirs_recursive_better([root.clone()]);
                info!("Scanned {} in {watch}", root.display());
                res
            })
            .unwrap();
        handles.push(handle);
    }
    let mut res_ok = Vec::new();
    let mut res_err = Vec::new();
    for handle in handles {
        let (cur_res_ok, cur_res_err) = handle.join().unwrap();
        // res_ok.extend(
        //     cur_res_ok
        //         .into_iter()
        //         .map(|v| v.to_str().unwrap().to_string()),
        // );
        res_ok.extend(cur_res_ok);
        res_err.extend(cur_res_err);
    }
    info!(
        "Scanned {} files with {} errors in {total_watch}",
        res_ok.len(),
        res_err.len()
    );
    for (path, err) in res_err {
        error!("failed {} because {}", path.display(), err);
    }

    res_ok
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
