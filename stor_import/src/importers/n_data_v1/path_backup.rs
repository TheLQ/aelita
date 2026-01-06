use crate::err::{StorImportErrorKind, StorImportResult};
use aelita_stor_diesel::RawDieselBytes;
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::ops::Range;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{CrashErrKind, RecursiveStatResult, ScanFileTypeWithPath, ScanStat};

type InputCacheData = (DiskScanFile, ScanStat);
type InputCacheDataRef<'s> = Vec<(DiskScanFile, &'s ScanStat)>;
const U64_BYTES: usize = 8;
const FLUSH_AT_ENTRIES: usize = 4096;

pub struct ChannelOutSaved {
    output: Vec<RecursiveStatResult>,
    cache_out: BufWriter<File>,
}

impl ChannelOutSaved {
    pub(crate) fn new(output_path: &Path) -> Self {
        Self {
            output: Vec::new(),
            cache_out: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(output_path)
                    .unwrap(),
            ),
        }
    }

    pub(crate) fn into_output(mut self) -> Vec<RecursiveStatResult> {
        let len = self.output.len();
        let mut remain = len % FLUSH_AT_ENTRIES;
        if remain != 0 {
            for value in (len - remain)..len {
                self.flush_values(value);
            }
        }
        self.output
    }

    pub(crate) fn push(&mut self, v: RecursiveStatResult) {
        self.output.push(v);
        self.flush_last_maybe();
    }

    fn flush_last_maybe(&mut self) {
        let len = self.output.len();
        if len == 0 {
            return;
        } else if len.is_multiple_of(FLUSH_AT_ENTRIES) {
            for value in (len - FLUSH_AT_ENTRIES)..len {
                self.flush_values(value)
            }
        }
    }

    // fn flush_values(&mut self, values_range: Range<usize>) {
    fn flush_values(&mut self, values: usize) {
        let (stype, scan) = &self.output[values];
        let safe_values: InputCacheData = (stype.into(), scan.clone());

        let encoded = RawDieselBytes::serialize_postcard(&safe_values)
            .unwrap_or_else(|e| panic!("failed {e} on value {values:?}"));
        let encoded_len = encoded.0.len();
        self.cache_out
            .write_all(&encoded_len.to_ne_bytes())
            .unwrap();
        self.cache_out.write_all(encoded.as_inner()).unwrap();
    }
}

//

pub fn read_input_cache(input_init: &[u8]) -> StorImportResult<Vec<InputCacheData>> {
    let mut res = Vec::new();
    let mut total_len = 0;
    let mut input = input_init;
    while !input.is_empty() {
        let (len_raw, remain) = input.split_at(U64_BYTES);
        let len = usize::from_ne_bytes(
            len_raw
                .as_array()
                .unwrap_or_else(|| {
                    panic!("unknown at {total_len} from input of {}", input_init.len())
                })
                .clone(),
        );

        let (data_raw, remain) = remain.split_at(len);
        let v: InputCacheData = postcard::from_bytes(data_raw).map_err(
            StorImportErrorKind::InvalidCompressedPaths.err_message_fn_map(|| {
                format!(
                    "bad postcard len {len} at {total_len} from input of {}",
                    input_init.len()
                )
            }),
        )?;
        // info!("decoded");

        // res.push((DiskScanFile::File { path: "fs".into() }, v));
        // res.push((v, ScanStat::dummy_value()));
        res.push(v);
        total_len += len + U64_BYTES;
        input = remain;
    }
    info!("finished decoding");

    Ok(res)
}

//

#[derive(Debug, Serialize, Deserialize)]
pub enum DiskScanFile {
    Dir { path: Vec<u8> },
    File { path: Vec<u8> },
    Symlink { path: Vec<u8>, target: Vec<u8> },
}

impl From<&ScanFileTypeWithPath> for DiskScanFile {
    fn from(value: &ScanFileTypeWithPath) -> Self {
        match value {
            ScanFileTypeWithPath::Dir { path } => Self::Dir {
                // path: path.as_os_str().to_os_string(),
                path: path.as_os_str().as_bytes().to_vec(),
            },
            ScanFileTypeWithPath::File { path } => Self::File {
                // path: path.as_os_str().to_os_string(),
                path: path.as_os_str().as_bytes().to_vec(),
            },
            ScanFileTypeWithPath::Symlink { path, target } => Self::Symlink {
                // path: path.as_os_str().to_os_string(),
                // target: target.as_os_str().to_os_string(),
                path: path.as_os_str().as_bytes().to_vec(),
                target: target.as_os_str().as_bytes().to_vec(),
            },
        }
    }
}

impl From<&DiskScanFile> for ScanFileTypeWithPath {
    fn from(value: &DiskScanFile) -> Self {
        match value {
            DiskScanFile::Dir { path } => Self::Dir {
                path: PathBuf::from(OsStr::from_bytes(path)),
                // path: PathBuf::from(path),
            },
            DiskScanFile::File { path } => Self::File {
                path: PathBuf::from(OsStr::from_bytes(path)),
                // path: PathBuf::from(path),
            },
            DiskScanFile::Symlink { path, target } => Self::Symlink {
                path: PathBuf::from(OsStr::from_bytes(path)),
                target: PathBuf::from(OsStr::from_bytes(target)),
                // path: PathBuf::from(path),
                // target: PathBuf::from(target),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::importers::n_data_v1::path_backup::{ChannelOutSaved, read_input_cache};
    use aelita_commons::log_init;
    use std::path::Path;
    use xana_commons_rs::tracing_re::info;
    use xana_commons_rs::{PrettyUnwrap, ScanFileTypeWithPath, ScanStat};

    #[test]
    fn end_to_end() {
        log_init();

        let output_path = Path::new("/tmp/path_backup_end_to_end.dat");
        let _ = std::fs::remove_file(output_path);
        {
            let mut out = ChannelOutSaved::new(output_path);
            out.push((
                ScanFileTypeWithPath::Dir {
                    path: "/lol".into(),
                },
                ScanStat::dummy_value(),
            ));
            out.push((
                ScanFileTypeWithPath::File {
                    path: "/lol/value".into(),
                },
                ScanStat::dummy_value(),
            ));
            out.into_output();
        }
        let saved_data = std::fs::read(output_path).unwrap();
        info!("saved data {}", saved_data.len());
        let res = read_input_cache(&saved_data).unwrap();
        println!("{res:?}")
    }
}
