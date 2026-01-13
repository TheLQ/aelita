use crate::api::assert_test_database;
use crate::api::bulk_insert::{BulkyInsert, DEFAULT_MEGA_CHUNK_SIZE, RowizerContext};
use crate::api::common::{SQL_PLACEHOLDER_MAX, check_insert_num_rows};
use crate::api::fancy_chunk::{Chunky, ChunkyAsRef, ChunkyPiece};
use crate::err::StorDieselErrorKind;
use crate::models::enum_types::ModelJournalTypeName;
use crate::schema_temp::{FAST_HD_COMPONENTS_CREATE, FAST_HD_COMPONENTS_TRUNCATE};
use crate::{
    CombinedStatAssociation, HdPathAssociation, ModelFileCompId, ModelFileTreeId, ModelJournalId,
    RawDieselBytes, StorIdTypeDiesel, components_get_from_fast, storapi_hd_get_path_by_path,
};
use crate::{StorDieselResult, StorTransaction, schema, schema_temp};
use crate::{build_associations_from_compressed, storapi_variables_get_str};
use chrono::NaiveDateTime;
use chrono::format::{DelayedFormat, StrftimeItems};
use diesel::prelude::*;
use diesel::{RunQueryDsl, dsl};
use std::collections::{HashMap, HashSet};
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{debug, info};
use xana_commons_rs::{BasicWatch, LOCALE, ResultXanaMap, io_op};
use xana_fs_indexer_rs::{CompressedPaths, ScanStat};

pub fn storapi_hd_tree_push(
    conn: &mut StorTransaction,
    compressed: CompressedPaths,
) -> StorDieselResult<()> {
    // diesel::sql_query("SET autocommit=0").execute(conn.inner())?;
    let autocommit = storapi_variables_get_str(conn.inner(), "autocommit")?;
    info!("autocommit is {autocommit}");

    let new = build_associations_from_compressed(conn, &compressed)?;

    let watch = BasicWatch::start();
    BulkyInsert {
        conn,
        table: "hd1_files_parents",
        keys: [
            "tree_id",
            "tree_depth",
            "component_id",
            "parent_id",
            "created",
            "modified",
            "size",
            "user_id",
            "group_id",
            "hard_links",
        ],
        values: &new,
        chunk_size: DEFAULT_MEGA_CHUNK_SIZE,
    }
    .insert_probably_huge_data(
        |RowizerContext {
             before,
             middle,
             after,
             output,
             row,
         }| {
            let (
                HdPathAssociation {
                    tree_id,
                    tree_depth,
                    component_id,
                    parent_id,
                },
                ScanStat {
                    created,
                    modified,
                    size,
                    user_id,
                    group_id,
                    hard_links,
                },
            ) = row;
            output.add_single_row(format_args!(
                "{before}\
                {tree_id}{middle}\
                {tree_depth}{middle}\
                {component_id}{middle}\
                {parent_id}{middle}\
                {created}{middle}\
                {modified}{middle}\
                {size}{middle}\
                {user_id}{middle}\
                {group_id}{middle}\
                {hard_links}\
                {after}",
                created = chrono_to_mysql(&created),
                modified = chrono_to_mysql(&modified),
                parent_id = match parent_id {
                    Some(v) => v.to_string(),
                    None => "NULL".into(),
                }
            ))
        },
    )?;
    debug!("inserted rows in {watch}");

    Ok(())
}

fn chrono_to_mysql(chron: &NaiveDateTime) -> DelayedFormat<StrftimeItems<'_>> {
    chron.format("'%Y-%m-%d %H:%M:%S%.6f'")
}

pub fn storapi_hd_tree_push_single(
    conn: &mut StorTransaction,
    parent: Option<ModelFileTreeId>,
    new_remain: &[(&[u8], ScanStat)],
) -> StorDieselResult<ModelFileTreeId> {
    let comps = components_upsert_cte(conn, &new_remain.iter().map(|v| &v.0).collect::<Vec<_>>())?;

    let mut next_new_id = schema::hd1_files_parents::table
        .select(dsl::max(schema::hd1_files_parents::tree_id))
        .execute(conn.inner())?;
    next_new_id += 1;

    let parent_depth = if let Some(parent) = parent {
        let depth = schema::hd1_files_parents::table
            .select(schema::hd1_files_parents::tree_depth)
            .filter(schema::hd1_files_parents::tree_id.eq(parent))
            .get_result::<u32>(conn.inner())?;
        Some(depth)
    } else {
        None
    };

    let mut new_parents = Vec::with_capacity(new_remain.len());
    let mut last_parent = parent;
    let mut next_tree_depth = if let Some(parent_depth) = parent_depth {
        parent_depth + 1
    } else {
        0
    };
    for (comp, stat) in new_remain {
        let tree_id = ModelFileTreeId::new_usize(next_new_id);
        new_parents.push(CombinedStatAssociation {
            path: HdPathAssociation {
                tree_id,
                parent_id: last_parent,
                component_id: *comps.get(*comp).unwrap(),
                tree_depth: next_tree_depth,
            },
            stat: stat.clone().into(),
        });
        last_parent = Some(tree_id);
        next_tree_depth += 1;
    }
    let rows = diesel::insert_into(schema::hd1_files_parents::table)
        .values(new_parents)
        .execute(conn.inner());
    check_insert_num_rows(rows, new_remain.len())?;

    Ok(last_parent.unwrap())
}

pub fn storapi_rebuild_parents(conn: &mut StorTransaction) -> StorDieselResult<()> {
    // diesel::sql_query("TRUNCATE TABLE `hd1_files_parents`").execute(conn.inner())?;

    // push_associations_fancy_insert(conn)?;
    let watch = BasicWatch::start();
    // let compressed_paths_raw = storapi_journal_get_data(conn, ModelJournalId::new(1))?;
    let compressed_paths_raw = io_op("journal-1.dat", |v| std::fs::read(v))
        .xana_err(StorDieselErrorKind::_TODO)
        .map(|v| RawDieselBytes(v))?;
    debug!(
        "loaded {} MB in {watch}",
        (compressed_paths_raw.as_inner().len() / 1000 / 1000).to_formatted_string(&LOCALE)
    );
    let watch = BasicWatch::start();
    let compressed: CompressedPaths = compressed_paths_raw.deserialize_postcard().unwrap();
    debug!("deserialized in {watch}");

    storapi_hd_tree_push(conn, compressed)?;
    Ok(())
}

pub fn components_upsert_cte(
    conn: &mut StorTransaction,
    components_unique_input: &[impl AsRef<[u8]>],
) -> StorDieselResult<HashMap<Vec<u8>, ModelFileCompId>> {
    components_update(conn, components_unique_input)?;
    components_get_from_fast(conn, components_unique_input)
}

fn components_update(
    conn: &mut StorTransaction,
    components_unique_input: &[impl AsRef<[u8]>],
) -> StorDieselResult<()> {
    // SQL cache of our millions of components
    diesel::sql_query(FAST_HD_COMPONENTS_CREATE).execute(conn.inner())?;
    diesel::sql_query(FAST_HD_COMPONENTS_TRUNCATE).execute(conn.inner())?;

    let watch = BasicWatch::start();
    let expected_length = components_unique_input.len();
    let components_unique = components_unique_input
        .iter()
        .map(|v| schema_temp::fast_hd_components::component.eq(v.as_ref()))
        .collect::<Vec<_>>();
    let mut total_rows = 0;
    for chunk in Chunky::ify(components_unique, "comps").pieces::<SQL_PLACEHOLDER_MAX>() {
        total_rows += diesel::insert_into(schema_temp::fast_hd_components::table)
            .values(chunk.as_ref())
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
        "INSERT INTO `hd1_files_components` (component) \
        SELECT component FROM `fast_hd_components` fast \
        WHERE \
        NOT EXISTS (\
            SELECT 1 FROM `hd1_files_components` WHERE fast.component = `hd1_files_components`.`component`\
        )",
    )
    .execute(conn.inner())?;

    info!(
        "added {} ({:.1}?) components in {watch}",
        rows.to_formatted_string(&LOCALE),
        rows / components_unique_input.len() * 100
    );

    Ok(())
}

/// Worse case duplicates 3 times pre-selecting all and failing, inserting all, then re-selecting
pub fn components_upsert_select_first<Comp>(
    conn: &mut StorTransaction,
    input: &[Comp],
) -> StorDieselResult<Vec<(ModelFileCompId, Vec<u8>)>>
where
    Comp: AsRef<[u8]>,
{
    let mut found = Vec::with_capacity(input.len());
    let mut missing: Vec<Vec<u8>> = Vec::new();
    for chunk in ChunkyAsRef::new(input, "comp_upsert_pre").pieces::<SQL_PLACEHOLDER_MAX>() {
        let rows = schema::hd1_files_components::table
            .filter(schema::hd1_files_components::component.eq_any(&chunk))
            .get_results::<(ModelFileCompId, Vec<u8>)>(conn.inner())?;
        if rows.len() != chunk.len() {
            let actual_comps: HashSet<&[u8]> =
                rows.iter().map(|(_, comp)| comp.as_slice()).collect();

            let mut expected_comps: HashSet<&[u8]> = HashSet::new();
            expected_comps.extend(chunk);

            let cur_missing: Vec<_> = expected_comps
                .difference(&actual_comps)
                .into_iter()
                .map(|v| v.to_vec())
                .collect();
            missing.extend(cur_missing);
        }
        found.extend(rows);
    }

    debug!("Inserting {} missing rows", missing.len());
    for missing in
        Chunky::ify(missing.as_slice(), "comp_upsert_insert").pieces::<SQL_PLACEHOLDER_MAX>()
    {
        let insert_rows: Vec<_> = missing
            .iter()
            .map(|v| schema::hd1_files_components::component.eq(v))
            .collect();
        let rows = diesel::insert_into(schema::hd1_files_components::table)
            .values(insert_rows)
            .execute(conn.inner());
        check_insert_num_rows(rows, missing.len())?;

        let new = schema::hd1_files_components::table
            .filter(schema::hd1_files_components::component.eq_any(missing))
            .get_results::<(ModelFileCompId, Vec<u8>)>(conn.inner())?;
        assert_eq!(new.len(), missing.len());
        found.extend(new);
    }

    Ok(found)
}

pub fn storapi_hd_links_add(
    conn: &mut StorTransaction,
    at: &[impl AsRef<[u8]>],
    target: &[impl AsRef<[u8]>],
) -> StorDieselResult<()> {
    let at_path = storapi_hd_get_path_by_path(conn, at)?;
    let target_path = storapi_hd_get_path_by_path(conn, target)?;

    let rows = diesel::insert_into(schema::hd1_files_links::table)
        .values((
            schema::hd1_files_links::at_tree.eq(at_path.last().unwrap()),
            schema::hd1_files_links::target_tree.eq(target_path.last().unwrap()),
        ))
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)?;
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

pub fn storapi_hd_parents_delete(conn: &mut StorTransaction) -> StorDieselResult<()> {
    assert_test_database(conn)?;
    diesel::sql_query("TRUNCATE TABLE `hd1_files_parents`").execute(conn.inner())?;
    Ok(())
}
