use crate::err::StorDieselErrorKind;
use crate::path_const::PathConst;
use crate::{HdPathDiesel, StorDieselResult, StorTransaction, storapi_row_count};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, CrashErrKind, LOCALE, SimpleIoMap};

/// effectively insert by 10-batch chain
/// with individual (id,parent_id) every value is duplicated twice
fn build_paths_mega_query(
    conn: &mut StorTransaction,
    paths: &[impl AsRef<Path>],
    component_to_id: &HashMap<String, u32>,
) -> StorDieselResult<()> {
    let watch = BasicWatch::start();

    const MIBI_BYTE: usize = 1024 * 1024;
    let max = 990 * MIBI_BYTE;
    let capacity = max + (5 * MIBI_BYTE);

    // so slow due to volume of placeholders for all 11 columns
    // instead safe to build query directly
    let mut total_inserted = 0;
    let mut path_iter = paths.iter().peekable();
    let mut i = 0;
    let mut query_values = String::with_capacity(capacity);
    while path_iter.peek().is_some() {
        let mut total_values = 0;
        query_values.truncate(0);
        query_values.push_str(
            "INSERT INTO `hd1_files_paths` \
             (`p0`, `p1`, `p2`, `p3`, `p4`, `p5`, `p6`, `p7`, `p8`, `p9`, `p10`) \
             VALUES ",
        );
        loop {
            if query_values.len() > max {
                break;
            }
            let path = match path_iter.next() {
                Some(path) => path.as_ref(),
                None => break,
            };
            let comp = HdPathDiesel::from_path(path, component_to_id)?;
            let [p0, p1, p2, p3, p4, p5, p6, p7, p8, p9, p10] = comp.into_array().map(|v| {
                if let Some(v) = v {
                    v.to_string()
                } else {
                    "NULL".into()
                }
            });
            write!(
                query_values,
                "({p0},{p1},{p2},{p3},{p4},{p5},{p6},{p7},{p8},{p9},{p10}),"
            )
            .unwrap();
            total_values += 1;
        }
        query_values.remove(query_values.len() - 1);

        trace!(
            "Insert chunk {i} - {} len {} MiB",
            total_values.to_formatted_string(&LOCALE),
            (query_values.len() / MIBI_BYTE).to_formatted_string(&LOCALE)
        );
        assert_eq!(query_values.capacity(), capacity);

        // total_inserted  +=
        conn.inner().batch_execute(&query_values)?;
        total_inserted += usize::try_from(storapi_row_count(conn)?)
            .map_err(StorDieselErrorKind::UnknownRowCount.err_map())?;

        i += 1;
    }

    trace!(
        "Inserted {} fast paths in {watch}",
        total_inserted.to_formatted_string(&LOCALE),
    );
    assert_eq!(total_inserted, paths.len());

    Ok(())
}

const IMPORT_COMPONENTS_PATH: PathConst = PathConst("import-data.temp.dat");
const ROW_SEP: u8 = 0x1e;
const COL_SEP: u8 = 0x1f;
fn build_paths_infile(
    conn: &mut StorTransaction,
    paths: &[impl AsRef<Path>],
    component_to_id: &HashMap<String, u32>,
) -> StorDieselResult<()> {
    let watch = BasicWatch::start();
    let mut content = Vec::new();
    for path in paths.iter() {
        let path = path.as_ref();
        let diesel_path = HdPathDiesel::from_path(path, &component_to_id)?;
        for field in diesel_path.into_array() {
            if let Some(v) = field {
                content.extend(v.to_string().as_bytes());
            }
            content.push(COL_SEP)
        }
        let content_len = content.len();
        content[content_len - 1] = ROW_SEP;
    }
    let import_path = IMPORT_COMPONENTS_PATH.as_ref();
    std::fs::write(import_path, &content)
        .map_io_err(import_path)
        .map_err(StorDieselErrorKind::LoadInfileFailed.err_map())?;
    info!("wrote to {} in {watch}", import_path.display());

    let import_path = import_path.canonicalize().unwrap();

    // let enabled = storapi_variables_get_str(conn.inner(), "local_infile")?;
    // info!("local_infile enabled {enabled}");
    //
    // // "SET GLOBAL local_infile=1;
    // dsl::sql_query("SET GLOBAL local_infile=ON").execute(conn.inner())?;
    //
    // let enabled = storapi_variables_get_str(conn.inner(), "local_infile")?;
    // info!("local_infile enabled {enabled}");

    let watch = BasicWatch::start();

    conn.inner().batch_execute(&format!(
        "LOAD DATA LOCAL INFILE '{}' \
        INTO TABLE `hd1_files_paths` \
        FIELDS TERMINATED BY {COL_SEP:#x} \
        LINES TERMINATED BY {ROW_SEP:#x} \
        (`p0`, `p1`, `p2`, `p3`, `p4`, `p5`, `p6`, `p7`, `p8`, `p9`, `p10`)",
        import_path.display()
    ))?;
    let rows = storapi_row_count(conn)?;
    info!(
        "wrote {} rows in {watch}",
        rows.to_formatted_string(&LOCALE)
    );

    Ok(())
}
