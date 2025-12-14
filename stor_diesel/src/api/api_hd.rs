use crate::api::common::{SQL_PLACEHOLDER_MAX, check_insert_num_rows};
use crate::id_types::{ModelJournalId, ModelJournalTypeName};
use crate::models::model_hd::{HD_PATH_DEPTH, HdPathDiesel, NewHdPathAssociation};
use crate::schema_temp::{
    FAST_HD_COMPONENTS_CREATE, FAST_HD_COMPONENTS_TRUNCATE, FAST_HD_PATHS_CREATE,
    FAST_HD_PATHS_TRUNCATE,
};
use crate::{StorDieselResult, StorTransaction, assert_test_database};
use crate::{schema, schema_temp};
use diesel::RunQueryDsl;
use diesel::connection::SimpleConnection;
use diesel::dsl::max;
use diesel::prelude::*;
use fxhash::FxHashSet;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, LOCALE};

pub fn storapi_hd_tree_push(
    conn: &mut StorTransaction,
    paths: &[impl AsRef<Path>],
) -> StorDieselResult<()> {
    let watch = BasicWatch::start();
    let components_unique_os: FxHashSet<&OsStr> =
        paths.iter().flat_map(|v| v.as_ref().iter()).collect();
    trace!(
        "Build {} components_unique in {watch}",
        components_unique_os.len()
    );
    let components_unique = components_unique_os
        .into_iter()
        .map(|v| v.to_str().unwrap())
        .collect::<Vec<_>>();

    // diesel::sql_query("SET autocommit=0").execute(conn.inner())?;

    components_update(conn, components_unique.as_slice())?;
    let component_to_id = components_get(conn, components_unique.as_slice())?;

    // push_associations_simple(conn, paths, component_to_id)?;
    push_associations_fancy(conn, paths, &component_to_id)?;

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
    }
    info!(
        "buffered {} fast components in {watch}",
        total_rows.to_formatted_string(&LOCALE)
    );
    assert_eq!(total_rows, expected_length);

    let watch = BasicWatch::start();
    let rows = diesel::sql_query(
        "INSERT INTO `hd1_files_tree` (component) \
        SELECT component FROM `fast_hd_components`\
        ON DUPLICATE KEY UPDATE `hd1_files_tree`.component = `fast_hd_components`.component",
    )
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

fn components_get(
    conn: &mut StorTransaction,
    components_unique_input: &[&str],
) -> StorDieselResult<HashMap<String, u32>> {
    let watch = BasicWatch::start();
    let lookup_vec: Vec<(Vec<u8>, u32)> =
        schema_temp::fast_hd_components::table
            .inner_join(schema::hd1_files_tree::table.on(
                schema::hd1_files_tree::component.eq(schema_temp::fast_hd_components::component),
            ))
            .select((
                schema::hd1_files_tree::component,
                schema::hd1_files_tree::id,
            ))
            .get_results(conn.inner())?;
    let lookup_map = lookup_vec
        .into_iter()
        .map(|(key, i)| (String::from_utf8(key).unwrap(), i))
        .collect::<HashMap<_, _>>();
    info!(
        "fetched {} rows in {watch}",
        lookup_map.len().to_formatted_string(&LOCALE)
    );
    assert_eq!(lookup_map.len(), components_unique_input.len());

    Ok(lookup_map)
}

/// Build parents tree locally without weird SQL shenanigans
fn push_associations_simple(
    conn: &mut StorTransaction,
    paths: &[impl AsRef<Path>],
    component_to_id: HashMap<String, u32>,
) -> StorDieselResult<()> {
    let mut associations = HashSet::new();
    for path in paths {
        let path = path.as_ref();
        let mut path_iter = path.iter();

        let mut prev = path_iter.next().unwrap().to_str().unwrap();
        while let Some(next_os) = path_iter.next() {
            let next = next_os.to_str().unwrap();
            associations.insert(NewHdPathAssociation {
                id: *component_to_id.get(next).unwrap(),
                parent_id: *component_to_id.get(prev).unwrap(),
            });
            prev = next;
        }
    }

    let watch = BasicWatch::start();
    let total_associations = associations.len();
    let mut total_inserted = 0;
    trace!("Inserting {total_associations} associations");
    let chunks = associations.iter().chunks(SQL_PLACEHOLDER_MAX / 2);
    let total_chunks = chunks.clone().into_iter().count() - 1;
    for (i, chunk) in chunks.into_iter().enumerate() {
        trace!("Insert chunk {i} of {total_chunks}");
        let chunk = chunk.collect::<Vec<_>>();

        total_inserted += diesel::insert_into(schema::hd1_files_parents::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(conn.inner())?;
    }
    info!("inserted {total_inserted} associations in {watch}");
    assert_eq!(total_inserted, total_associations);

    Ok(())
}

// todo: inserting so many
fn push_associations_fancy(
    conn: &mut StorTransaction,
    paths: &[impl AsRef<Path>],
    component_to_id: &HashMap<String, u32>,
) -> StorDieselResult<()> {
    diesel::sql_query(FAST_HD_PATHS_CREATE).execute(conn.inner())?;
    diesel::sql_query(FAST_HD_PATHS_TRUNCATE).execute(conn.inner())?;

    let watch = BasicWatch::start();
    let mut total_inserted = 0;
    let chunks = paths.chunks(SQL_PLACEHOLDER_MAX / HD_PATH_DEPTH);
    let total_chunks = chunks.len() - 1;
    for (i, chunk) in chunks.into_iter().enumerate() {
        trace!("Insert chunk {i} of {total_chunks}");
        let fast_values = chunk
            .iter()
            .map(|v| HdPathDiesel::from_path(v.as_ref(), component_to_id))
            .collect::<Vec<_>>();
        total_inserted += diesel::insert_into(schema_temp::fast_hd_paths::table)
            .values(fast_values)
            .execute(conn.inner())?;
    }
    assert_eq!(total_inserted, paths.len());
    trace!(
        "Inserted {} fast paths in {watch}",
        total_inserted.to_formatted_string(&LOCALE),
    );

    total_inserted = 0;
    for i in 0..(HD_PATH_DEPTH - 1) {
        let next_i = i + 1;
        total_inserted += diesel::sql_query(format!(
            "INSERT INTO `hd1_files_parents` (id, parent_id) SELECT p{i} as parent_id, p{next_i} as id FROM `fast_hd_paths`"
        )).execute(conn.inner())?;
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

// struct Chunkify<V>(V);
//
// impl<V> Iterator for Chunkify<V>
// where V: Iterator
// {
//     type Item = <V as Iterator>::Item;
//
//     fn next(&mut self) -> Option<Self::Item> {
//
//     }
// }

// fn __something_query() {
//     for chunk in paths.chunks(SQL_PLACEHOLDER_MAX) {
//         let query_raw = format!(
//             "SELECT \
//             {selects} \
//             FROM `hd1_files_tree` as tree \
//             INNER JOIN `hd1_files_parents` j0 ON tree.id = j0.id
//             {joins} \
//             WHERE \
//             tree.name = IN({ins}) AND \
//             j0.parentId = NULL",
//             selects = (0..HD_PATH_DEPTH)
//                 .map(|i| format!("j{i}.name as p{i}"))
//                 .collect::<CommaJoiner>(),
//             joins = (1..HD_PATH_DEPTH)
//                 .map(|i| {
//                     let prev_i = i - 1;
//                     format!("LEFT JOIN `hd1_files_parents` as j{i} on j{i}.parentId = j{prev_i}.id")
//                 })
//                 .collect::<SpaceJoiner>(),
//             ins = (0..chunk.len()).map(|_| "?").collect::<CommaJoiner>()
//         );
//
//         let mut query = diesel::sql_query(query_raw).into_boxed();
//         for path in chunk {
//             let path = path.as_ref();
//             let root_component = path.iter().next().unwrap();
//             query = query.bind::<Text, _>(root_component.to_str().unwrap());
//         }
//
//         let result = query.load::<HdPathDiesel>(conn.inner())?;
//         if result.len() != chunk.len() {
//             return Err(StorDieselError::query_fail(format!(
//                 "unbalanced result {} to chunk {}",
//                 result.len(),
//                 chunk.len()
//             )));
//         }
//     }
//
//     Ok(todo!())
// }

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
