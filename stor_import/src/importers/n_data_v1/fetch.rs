use crate::err::StorImportResult;
use aelita_stor_diesel::ModelJournalTypeName;
use aelita_stor_diesel::NewModelJournalImmutable;
use aelita_stor_diesel::path_const::PathConst;
use aelita_stor_diesel::storapi_journal_immutable_push_single;
use aelita_stor_diesel::{CompressedPaths, RawDieselBytes};
use aelita_stor_diesel::{StorDieselError, StorTransaction};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::thread;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{error, info};
use xana_commons_rs::{BasicWatch, LOCALE, SimpleIoMap, read_dirs_recursive_better};

static ROOTS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let path = Path::new("ndata_roots.txt");
    let raw = std::fs::read_to_string(path).map_io_err(path).unwrap();
    raw.split('\n').map(|s| s.to_string()).collect()
});
pub const COMPRESSEDD_CACHE: PathConst = PathConst("compressed_paths.cache.json");

pub fn paths_load() -> Vec<PathBuf> {
    let total_watch = BasicWatch::start();
    let mut handles = Vec::new();
    for root in ROOTS.iter() {
        let root = Path::new(root).to_path_buf();
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

fn paths_compressed() -> StorImportResult<(CompressedPaths, Vec<u8>)> {
    let paths_raw = paths_load();
    let compressed =
        CompressedPaths::from_paths(&paths_raw).map_err(StorDieselError::query_fail)?;
    let encoded_diesel = RawDieselBytes::serialize_postcard(&compressed)?;
    let encoded = encoded_diesel.as_inner();

    let raw_size: usize = paths_raw.iter().map(|v| v.to_str().unwrap().len()).sum();
    let encoded_size: usize = encoded.len();

    let saved_bytes = raw_size as isize - encoded_size as isize;
    let saved_percent = (encoded_size as f64 / raw_size as f64) * 100.0;
    info!(
        "encoded {} raw {} raw {} reduction {saved_percent:.1}%",
        encoded_size.to_formatted_string(&LOCALE),
        raw_size.to_formatted_string(&LOCALE),
        saved_bytes.to_formatted_string(&LOCALE)
    );

    let cache = Path::new("compressed_paths.cache.json");
    std::fs::write(cache, &encoded).map_io_err(cache)?;
    info!("wrote to {}", cache.display());

    Ok((compressed, encoded_diesel.into_inner()))
}

pub fn storfetch_ndata(conn: &mut StorTransaction) -> StorImportResult<()> {
    let (_compressed, compressed_encoded) = paths_compressed()?;
    // let compressed_encoded = std::fs::read(COMPRESSEDD_CACHE).map_io_err(COMPRESSEDD_CACHE)?;
    let data = RawDieselBytes(compressed_encoded);

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
