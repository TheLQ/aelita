use std::path::{Path, PathBuf};
use std::thread;
use xana_commons_rs::tracing_re::{error, info};
use xana_commons_rs::{BasicWatch, read_dirs_recursive_better};

pub fn storfetch_ndata_pre(roots: impl IntoIterator<Item = impl AsRef<Path>>) -> Vec<PathBuf> {
    let total_watch = BasicWatch::start();
    let mut handles = Vec::new();
    for root in roots {
        let root = root.as_ref();
        info!("scanning {}...", root.display());
        let root = root.to_owned();
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
