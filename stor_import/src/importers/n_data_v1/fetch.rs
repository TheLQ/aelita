use crate::importers::n_data_v1::defs::CompressedPaths;
use aelita_stor_diesel::api_journal::storapi_journal_immutable_push_single;
use aelita_stor_diesel::id_types::ModelJournalTypeName;
use aelita_stor_diesel::model_journal::NewModelJournalDataImmutable;
use aelita_stor_diesel::util_types::RawDieselBytes;
use aelita_stor_diesel::{StorDieselError, StorDieselResult, StorTransaction};
use std::cell::LazyCell;
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

fn paths_compressed() -> StorDieselResult<(CompressedPaths, String)> {
    let paths_raw = paths_load();
    let compressed =
        CompressedPaths::from_paths(&paths_raw).map_err(StorDieselError::query_fail)?;
    let compressed_json = serde_json::to_string(&compressed)?;

    let paths_raw_size: usize = paths_raw.iter().map(|v| v.to_str().unwrap().len()).sum();
    let compressed_json_size: usize = compressed_json.len();

    let saved_bytes = paths_raw_size as isize - compressed_json_size as isize;
    let saved_percent = (1.0 - (compressed_json_size as f64 / paths_raw_size as f64)) * 100.0;
    info!(
        "output {} compressed {} raw {} reduction {saved_percent:.1}%",
        compressed_json_size.to_formatted_string(&LOCALE),
        paths_raw_size.to_formatted_string(&LOCALE),
        saved_bytes.to_formatted_string(&LOCALE)
    );
    Ok((compressed, compressed_json))
}

pub fn storfetch_ndata(conn: &mut StorTransaction) -> StorDieselResult<()> {
    let (_compressed, compressed_json) = paths_compressed()?;
    let data = RawDieselBytes::new(compressed_json.into_bytes());

    let journal_id = storapi_journal_immutable_push_single(
        conn,
        NewModelJournalDataImmutable {
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
