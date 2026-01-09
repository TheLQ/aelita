use crate::api::api_hd::components_get_from_fast;
use crate::api::assert_test_database;
use crate::api::bulk_insert::bulk_insert;
use crate::api::common::SQL_PLACEHOLDER_MAX;
use crate::models::enum_types::ModelJournalTypeName;
use crate::schema_temp::{FAST_HD_COMPONENTS_CREATE, FAST_HD_COMPONENTS_TRUNCATE};
use crate::{HdPathAssociation, ModelFileCompId, ModelJournalId};
use crate::{ScanStatDiesel, StorIdTypeDiesel, storapi_journal_get_data};
use crate::{StorDieselResult, StorTransaction, schema, schema_temp};
use crate::{build_associations_from_compressed, storapi_variables_get_str};
use chrono::NaiveDateTime;
use chrono::format::{DelayedFormat, StrftimeItems};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use std::collections::HashMap;
use std::fmt::Write;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{debug, info, trace};
use xana_commons_rs::{BasicWatch, LOCALE};
use xana_fs_indexer_rs::{CompressedPaths, ScanStat};

pub fn storapi_hd_tree_push(
    conn: &mut StorTransaction,
    compressed: CompressedPaths,
) -> StorDieselResult<()> {
    // diesel::sql_query("SET autocommit=0").execute(conn.inner())?;
    let autocommit = storapi_variables_get_str(conn.inner(), "autocommit")?;
    info!("autocommit is {autocommit}");

    let new = build_associations_from_compressed(conn, &compressed)?;

    let rows = bulk_insert(
        conn,
        "hd1_files_parents",
        [
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
        &new,
        |(
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
        ),
         query_res| {
            write!(
                query_res,
                "{tree_id},{tree_depth},{component_id},{parent_id},{created},{modified},{size},{user_id},{group_id},{hard_links}",
                created = chrono_to_mysql(created),
                modified = chrono_to_mysql(modified),
                parent_id = match parent_id {
                   Some(v) => v.to_string(),
                   None => "NULL".into(),
                }
            ).unwrap()
        },
        4_000_000,
    )?;
    debug!("inserted {rows} rows");

    Ok(())
}

fn chrono_to_mysql(chron: &NaiveDateTime) -> DelayedFormat<StrftimeItems> {
    chron.format("'%Y-%m-%d %H:%M:%S%.6f'")
}

pub fn storapi_rebuild_parents(conn: &mut StorTransaction) -> StorDieselResult<()> {
    // diesel::sql_query("TRUNCATE TABLE `hd1_files_parents`").execute(conn.inner())?;

    // push_associations_fancy_insert(conn)?;
    let watch = BasicWatch::start();
    let compressed_paths_raw = storapi_journal_get_data(conn, ModelJournalId::new(148))?;
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

pub fn storapi_hd_components_get_or_insert(
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
