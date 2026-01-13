use crate::err::StorDieselErrorKind;
use crate::{RawDieselBytes, StorDieselResult};
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, LOCALE, ResultXanaMap, SimpleIoMap};
use xana_fs_indexer_rs::CompressedPaths;

pub fn encode_compressed_paths(
    compressed: &CompressedPaths,
    raw_size: Option<usize>,
) -> StorDieselResult<Vec<u8>> {
    let postcard_size_i;
    let compressed_size_i;
    let encoded = {
        let watch = BasicWatch::start();
        let post =
            RawDieselBytes::serialize_postcard(&compressed).xana_err(StorDieselErrorKind::_TODO)?;
        postcard_size_i = post.0.len() as isize;
        trace!("Postcard serialized in {watch}");

        let watch = BasicWatch::start();
        let real = zstd::encode_all(post.as_inner(), 0)
            .map_io_err("zstd-err")
            .xana_err(StorDieselErrorKind::_TODO)?;
        compressed_size_i = real.len() as isize;
        trace!("ZFS serialized in {watch}");
        real
    };

    let postcard_size_f = postcard_size_i as f64;
    let compressed_size_f = compressed_size_i as f64;
    let common_width = 14;

    let mut diff_i;
    let mut percent;
    info!(
        "zstd     {:>common_width$}",
        compressed_size_i.to_formatted_string(&LOCALE)
    );
    info!(
        "postcard {:>common_width$}",
        postcard_size_i.to_formatted_string(&LOCALE),
    );
    if let Some(raw_size) = raw_size {
        info!(
            "raw      {:>common_width$}",
            raw_size.to_formatted_string(&LOCALE),
        );
    }
    diff_i = postcard_size_i - compressed_size_i;
    percent = (compressed_size_f / postcard_size_f) * 100.0;
    info!(
        " - post diff {:>common_width$}  reduced to % {:.1}",
        diff_i.to_formatted_string(&LOCALE),
        percent
    );
    if let Some(raw_size) = raw_size {
        let raw_size_i = raw_size as isize;
        let raw_size_f = raw_size as f64;
        diff_i = raw_size_i - compressed_size_i;
        percent = (compressed_size_f / raw_size_f) * 100.0;
        info!(
            " - raw diff {:>common_width$}  reduced to % {:.1}",
            diff_i.to_formatted_string(&LOCALE),
            percent
        );
    }

    Ok(encoded)
}
