use crate::api::common::{SQL_PLACEHOLDER_MAX, check_insert_num_rows};
use crate::models::model_hd::{HD_PATH_DEPTH, HdPathDiesel};
use crate::schema_temp::{SQL_FAST_HD_CREATE, SQL_FAST_HD_DROP};
use crate::{StorDieselError, schema, schema_temp};
use crate::{StorDieselResult, StorTransaction};
use diesel::RunQueryDsl;
use diesel::dsl::max;
use diesel::prelude::*;
use diesel::query_builder::Query;
use diesel::sql_types::Text;
use fxhash::FxHashSet;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, CommaJoiner, LOCALE, SpaceJoiner};

pub fn storapi_hd_tree_push(
    conn: &mut StorTransaction,
    paths: &[impl AsRef<Path>],
) -> StorDieselResult<()> {
    let watch = BasicWatch::start();
    let mut components_unique: FxHashSet<&[u8]> = paths
        .iter()
        .flat_map(|v| v.as_ref().iter())
        .map(|v| v.to_str().unwrap().as_bytes())
        .collect::<FxHashSet<_>>();
    trace!(
        "Build {} components_unique in {watch}",
        components_unique.len()
    );

    /*
    // build_temp_lookup(conn, &components_unique)?;
    // diesel::insert_into(schema::hd1_files_tree::table)
    //     .values(diesel::select(schema_temp::fast_hd_data::table))
    //     .execute(conn.inner())?;
    let rows = diesel::sql_query("INSERT INTO `hd1_files_tree` SELECT * from `fast_hd_data`")
        .execute(conn.inner())?;
    info!("Inserted new rows {}", rows.to_formatted_string(&LOCALE));
     */
    push_missing_components(conn, &components_unique)?;

    push_missing_associations_simple(conn, paths, &components_unique)?;

    // let rows = diesel::sql_query(SQL_FAST_HD_DROP).execute(conn.inner());
    // check_insert_num_rows(rows, 0)?;

    Ok(())
}

fn build_temp_lookup(
    conn: &mut StorTransaction,
    components_unique: &HashSet<&[u8]>,
) -> StorDieselResult<()> {
    let rows = diesel::sql_query(SQL_FAST_HD_CREATE).execute(conn.inner());
    check_insert_num_rows(rows, 0)?;

    let watch = BasicWatch::start();
    let mut total_inserted = 0;
    for chunk in &components_unique.iter().chunks(SQL_PLACEHOLDER_MAX / 2) {
        let values = chunk
            .map(|v| schema_temp::fast_hd_data::component.eq(v))
            .collect::<Vec<_>>();
        total_inserted += diesel::insert_into(schema_temp::fast_hd_data::table)
            .values(values)
            .execute(conn.inner())?;
    }
    assert_eq!(total_inserted, components_unique.len());
    trace!(
        "Inserted {} temp components in {watch}",
        total_inserted.to_formatted_string(&LOCALE),
    );

    Ok(())
}

fn push_missing_components(
    conn: &mut StorTransaction,
    components_unique_input: &FxHashSet<&[u8]>,
) -> StorDieselResult<()> {
    // let components_existing = schema::hd1_files_tree::table
    //     .select(schema::hd1_files_tree::component)
    //     .filter(schema::hd1_files_tree::component.eq_any(components_unique_input))
    //     .get_results::<String>(conn.inner())?;
    // if components_existing.len() == components_unique_input.len() {
    //     // every component exists wow
    //     return Ok(());
    // }
    //
    // let mut components_unique = components_unique_input.clone();
    // for component in &components_existing {
    //     components_unique.remove(component.as_str());
    // }
    //
    ////////////////////////////////////
    let watch = BasicWatch::start();
    let new_components = components_unique_input
        .into_iter()
        .map(|v| schema::hd1_files_tree::component.eq(v))
        .collect::<Vec<_>>();
    let chunks = new_components.chunks(SQL_PLACEHOLDER_MAX / 2);
    let mut total_inserted = 0;
    for chunk in chunks {
        total_inserted += diesel::insert_into(schema::hd1_files_tree::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(conn.inner())?;
    }
    trace!(
        "Inserted {} of {} ({:.1}%) components in {watch}",
        total_inserted,
        components_unique_input.len(),
        total_inserted / components_unique_input.len() * 100
    );

    Ok(())
}

// fn get_existing_ids(
//     conn: &mut StorTransaction,
//     components_unique_input: &HashSet<&str>,
// ) -> StorDieselResult<Vec<String>> {
//     schema::hd1_files_tree::table
//         .select(schema::hd1_files_tree::component)
//         .filter(schema::hd1_files_tree::component.eq_any(components_unique_input))
//         .get_results::<String>(conn.inner())
//         .map_err(Into::into)
// }

/// Build parents tree locally without weird SQL shenanigans
fn push_missing_associations_simple(
    conn: &mut StorTransaction,
    paths: &[impl AsRef<Path>],
    components_unique_input: &FxHashSet<&[u8]>,
) -> StorDieselResult<()> {
    let mut components_ids: HashMap<Vec<u8>, u32> = HashMap::new();
    for chunk in &components_unique_input
        .iter()
        .chunks(SQL_PLACEHOLDER_MAX / 2)
    {
        let rows = schema::hd1_files_tree::table
            .select((
                schema::hd1_files_tree::component,
                schema::hd1_files_tree::id,
            ))
            .filter(schema::hd1_files_tree::component.eq_any(chunk))
            .get_results(conn.inner())?;
        components_ids.extend(rows);
    }
    assert_eq!(components_ids.len(), components_unique_input.len());
    trace!("Fetched {} components for comparison", components_ids.len());

    let mut associations = Vec::new();
    for path in paths {
        let path = path.as_ref();
        let mut path_iter = path.iter();

        let mut prev: &[u8] = path_iter.next().unwrap().as_bytes();
        while let Some(next_os) = path_iter.next() {
            let next: &[u8] = next_os.as_bytes();
            associations.push((
                schema::hd1_files_parents::id.eq(components_ids.get(next).unwrap()),
                schema::hd1_files_parents::parentId.eq(components_ids.get(prev).unwrap()),
            ));
            prev = next;
        }
    }

    let total_associations = associations.len();
    let mut total_inserted = 0;
    trace!("Inserting {total_associations} associations");
    for chunk in associations.chunks(SQL_PLACEHOLDER_MAX / 2) {
        total_inserted += diesel::insert_into(schema::hd1_files_parents::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(conn.inner())?;
    }
    assert_eq!(total_inserted, total_associations);

    Ok(())
}

fn push_missing_associations_todo(
    conn: &mut StorTransaction,
    paths: &[impl AsRef<Path>],
) -> StorDieselResult<Vec<u32>> {
    for chunk in paths.chunks(SQL_PLACEHOLDER_MAX) {
        let query_raw = format!(
            "SELECT \
            {selects} \
            FROM `hd1_files_tree` as tree \
            INNER JOIN `hd1_files_parents` j0 ON tree.id = j0.id
            {joins} \
            WHERE \
            tree.name = IN({ins}) AND \
            j0.parentId = NULL",
            selects = (0..HD_PATH_DEPTH)
                .map(|i| format!("j{i}.name as p{i}"))
                .collect::<CommaJoiner>(),
            joins = (1..HD_PATH_DEPTH)
                .map(|i| {
                    let prev_i = i - 1;
                    format!("LEFT JOIN `hd1_files_parents` as j{i} on j{i}.parentId = j{prev_i}.id")
                })
                .collect::<SpaceJoiner>(),
            ins = (0..chunk.len()).map(|_| "?").collect::<CommaJoiner>()
        );

        let mut query = diesel::sql_query(query_raw).into_boxed();
        for path in chunk {
            let path = path.as_ref();
            let root_component = path.iter().next().unwrap();
            query = query.bind::<Text, _>(root_component.to_str().unwrap());
        }

        let result = query.load::<HdPathDiesel>(conn.inner())?;
        if result.len() != chunk.len() {
            return Err(StorDieselError::query_fail(format!(
                "unbalanced result {} to chunk {}",
                result.len(),
                chunk.len()
            )));
        }
    }

    Ok(todo!())
}
