use crate::api::common::SQL_PLACEHOLDER_MAX;
use crate::api_hd::components_get;
use crate::api_variables::{storapi_row_count, storapi_variables_get_str};
use crate::id_types::{ModelJournalId, ModelJournalTypeName};
use crate::model_hd::{HD_PATH_DEPTH, HdPathDiesel};
use crate::path_const::PathConst;
use crate::schema_temp::{FAST_HD_COMPONENTS_CREATE, FAST_HD_COMPONENTS_TRUNCATE};
use crate::{StorDieselResult, StorTransaction, assert_test_database, schema, schema_temp};
use diesel::RunQueryDsl;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use fxhash::FxHashSet;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::Path;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, CommaJoiner, LOCALE, SimpleIoMap, SpaceJoiner};

pub fn storapi_hd_tree_push(
    conn: &mut StorTransaction,
    paths: &[impl AsRef<Path>],
) -> StorDieselResult<()> {
    trace!("unique-ifying...");
    let watch = BasicWatch::start();
    let components_unique_os: FxHashSet<&OsStr> =
        paths.iter().flat_map(|v| v.as_ref().iter()).collect();
    let components_unique = components_unique_os
        .into_iter()
        .map(|v| v.to_str().unwrap())
        .collect::<Vec<_>>();
    trace!(
        "Build {} components_unique in {watch}",
        components_unique.len()
    );

    // diesel::sql_query("SET autocommit=0").execute(conn.inner())?;
    let autocommit = storapi_variables_get_str(conn.inner(), "autocommit")?;
    info!("autocommit is {autocommit}");

    components_update(conn, components_unique.as_slice())?;
    let component_to_id = components_get(conn, components_unique.as_slice())?;

    // push_associations_simple(conn, paths, component_to_id)?;
    // push_associations_fancy(conn, paths, &component_to_id)?;

    diesel::sql_query("TRUNCATE TABLE `hd1_files_paths`").execute(conn.inner())?;

    // build_paths_mega_query(conn, paths, &component_to_id)?;
    build_paths_infile(conn, paths, &component_to_id)?;

    push_associations_fancy_insert(conn)?;

    Ok(())
}

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
        total_inserted += usize::try_from(storapi_row_count(conn)?)?;

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
    std::fs::write(import_path, &content).map_io_err(import_path)?;
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

pub fn storapi_rebuild_parents(conn: &mut StorTransaction) -> StorDieselResult<()> {
    diesel::sql_query("TRUNCATE TABLE `hd1_files_parents`").execute(conn.inner())?;
    push_associations_fancy_insert(conn)?;
    Ok(())
}

fn push_associations_fancy_insert(conn: &mut StorTransaction) -> StorDieselResult<()> {
    let watch = BasicWatch::start();
    let mut total_inserted = 0;
    total_inserted += diesel::sql_query(
        "INSERT IGNORE INTO `hd1_files_parents` (component_id, tree_depth) \
        ( SELECT DISTINCT p0 as component_id, 0 FROM `hd1_files_paths` )",
    )
    .execute(conn.inner())?;
    trace!("inital inserted {total_inserted}");

    for comp_i in 1..(HD_PATH_DEPTH - 1) {
        let next_comp_i = comp_i + 1;
        let prev_comp_i = comp_i - 1;
        let p_cols = (0..=comp_i)
            .map(|i| format!("p{i}"))
            .collect::<CommaJoiner>();
        let joins = (1..comp_i)
            .map(|i| {
                format!(
                    "INNER JOIN hd1_files_parents parents{i} ON \
                    parents{i}.tree_depth = {i} AND \
                    parents{i}.component_id = paths.p{i} AND \
                    parents{i}.parent_id = parents{prev_i}.tree_id",
                    prev_i = i - 1
                )
            })
            .collect::<SpaceJoiner>();
        let cur_inserted = diesel::sql_query(format!(
            "INSERT IGNORE INTO `hd1_files_parents` (component_id, parent_id, tree_depth) (\
            SELECT DISTINCT paths.p{comp_i}, parents{prev_comp_i}.tree_id, {comp_i}  \
            FROM `hd1_files_paths` paths \
            INNER JOIN hd1_files_parents parents0 ON \
                parents0.tree_depth = 0 AND \
                parents0.component_id = paths.p0 AND \
                parents0.parent_id IS NULL \
            {joins} \
            WHERE \
            p{comp_i} IS NOT NULL\
            )"
        ))
        // ON DUPLICATE KEY UPDATE `hd1_files_parents`.id = `hd1_files_parents`.id"
        .execute(conn.inner())?;
        trace!("inserted {cur_inserted} rows");
        total_inserted += cur_inserted;
    }
    trace!(
        "Inserted {} fast paths in {watch}",
        total_inserted.to_formatted_string(&LOCALE),
    );

    // let selects = (0..(HD_PATH_DEPTH - 1))
    //     .map(|i| {
    //         let next_i = i + 1;
    //         format!("SELECT p{i} as parent_id, p{next_i} as id FROM `fast_hd_paths`")
    //     })
    //     .collect::<Vec<_>>()
    //     .join(" UNION ALL ");
    // let query_raw = format!("INSERT INTO `hd1_files_parents` (id, parent_id) ({selects})");
    // let rows = diesel::sql_query(query_raw).execute(conn.inner())?;
    // trace!(
    //     "Inserted {} fast paths in {watch}",
    //     rows.to_formatted_string(&LOCALE),
    // );
    Ok(())
}

fn components_update(
    conn: &mut StorTransaction,
    components_unique_input: &[&str],
) -> StorDieselResult<()> {
    // SQL cache of our millions of components
    diesel::sql_query(FAST_HD_COMPONENTS_CREATE).execute(conn.inner())?;
    diesel::sql_query(FAST_HD_COMPONENTS_TRUNCATE).execute(conn.inner())?;

    let watch = BasicWatch::start();
    let expected_length = components_unique_input.len();
    let components_unique = components_unique_input
        .iter()
        .map(|v| schema_temp::fast_hd_components::component.eq(v.as_bytes()))
        .collect::<Vec<_>>();
    let mut total_rows = 0;
    let chunks = components_unique.chunks(SQL_PLACEHOLDER_MAX);
    let total_chunks = chunks.len();
    for (i, chunk) in chunks.enumerate() {
        trace!("Insert chunk {i} of {total_chunks}");
        total_rows += diesel::insert_into(schema_temp::fast_hd_components::table)
            .values(chunk)
            .execute(conn.inner())?;
        // if 1 + 1 == 2 {
        //     break;
        // }
    }
    info!(
        "buffered {} fast components in {watch}",
        total_rows.to_formatted_string(&LOCALE)
    );
    assert_eq!(total_rows, expected_length);

    let watch = BasicWatch::start();
    let rows = diesel::sql_query(
        "INSERT IGNORE INTO `hd1_files_components` (component) \
        SELECT component FROM `fast_hd_components`",
    )
    // ON DUPLICATE KEY UPDATE `hd1_files_tree`.component = `hd1_files_tree`.component
    .execute(conn.inner())?;
    // :-(
    // let rows = diesel::insert_into(schema::hd1_files_tree::table)
    //     .values(schema::hd1_files_tree::table.as_sql())
    //     .execute(conn.inner())?;
    info!(
        "added {} ({:.1}?) components in {watch}",
        rows.to_formatted_string(&LOCALE),
        rows / components_unique_input.len() * 100
    );

    Ok(())
}

pub fn storapi_hd_revert_by_pop(conn: &mut StorTransaction) -> StorDieselResult<()> {
    assert_test_database(conn)?;

    let last_journal: Vec<(ModelJournalId, ModelJournalTypeName)> =
        schema::journal_immutable::table
            .select((
                schema::journal_immutable::journal_id,
                schema::journal_immutable::journal_type,
            ))
            .filter(schema::journal_immutable::committed.eq(true))
            .order_by(schema::journal_immutable::journal_id.desc())
            .get_results(conn.inner())?;
    let to_reset = last_journal
        .into_iter()
        .take_while(|(_, journal_type)| *journal_type == ModelJournalTypeName::NData1)
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    let rows = diesel::update(schema::journal_immutable::table)
        .filter(schema::journal_immutable::journal_id.eq_any(to_reset))
        .set(schema::journal_immutable::committed.eq(false))
        .execute(conn.inner())?;
    info!("un-commit {rows} ndata rows");

    let rows = diesel::sql_query("TRUNCATE TABLE `hd1_files_parents`").execute(conn.inner())?;
    info!("truncate {rows} rows");

    Ok(())
}
