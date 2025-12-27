use crate::{HD_PATH_DEPTH, StorDieselResult, StorTransaction, path_components};
use diesel::sql_types::Binary;
use diesel::{QueryableByName, RunQueryDsl};
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::trace;
use xana_commons_rs::{BasicWatch, CommaJoiner, LOCALE, SpaceJoiner};

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

fn hd_list_children_paths(
    conn: &mut StorTransaction,
    path: impl AsRef<Path>,
) -> StorDieselResult<Vec<String>> {
    let path = path.as_ref();
    let path_components = path_components(path, |c| c.as_bytes())?;

    let selected_column = path_components.len();
    let mut query_builder = format!(
        "SELECT `hd1_files_components`.component FROM ( \
            SELECT DISTINCT(p{selected_column}) as child \
            FROM `hd1_files_paths` \
            WHERE "
    );

    for i in 0..path_components.len() {
        query_builder.push_str(&format!(
            "p{i} = ( \
            SELECT `hd1_files_components`.`id` \
            FROM `hd1_files_components` \
            WHERE `hd1_files_components`.`component` = ? \
            LIMIT 1 \
            ) AND "
        ));
    }
    query_builder.truncate(query_builder.len() - 4);

    query_builder.push_str(
        ") AS path_ids \
        INNER JOIN `hd1_files_components`\
        ON `hd1_files_components`.id = `path_ids`.child",
    );

    #[derive(QueryableByName)]
    struct PathResult {
        #[diesel(sql_type = diesel::sql_types::Text)]
        component: String,
    }

    let mut query = diesel::sql_query(query_builder).into_boxed();
    for component in path_components {
        query = query.bind::<Binary, _>(component.to_vec());
    }
    let rows = query.get_results::<PathResult>(conn.inner())?;
    let res = rows.into_iter().map(|v| v.component).collect();

    // todo Diesel doesn't support SELECT component FROM (SELECT distinct(field)))
    // let mut query = schema::hd1_files_paths::table
    //     .select(sql::<Unsigned<Integer>>(&format!(
    //         "DISTINCT(p{})",
    //         path_components.len()
    //     )))
    //     .into_boxed();
    // for (i, part) in path_components.iter().enumerate() {
    //     let content = part; //.to_vec();
    //     let query_component_to_id = schema::hd1_files_components::table
    //         .select(schema::hd1_files_components::id)
    //         .filter(schema::hd1_files_components::component.eq(content))
    //         .single_value();
    //     match i {
    //         0 => query = query.filter(schema::hd1_files_paths::p0.eq(query_component_to_id)),
    //         1 => query = query.filter(schema::hd1_files_paths::p1.eq(query_component_to_id)),
    //         2 => query = query.filter(schema::hd1_files_paths::p2.eq(query_component_to_id)),
    //         3 => query = query.filter(schema::hd1_files_paths::p3.eq(query_component_to_id)),
    //         4 => query = query.filter(schema::hd1_files_paths::p4.eq(query_component_to_id)),
    //         5 => query = query.filter(schema::hd1_files_paths::p5.eq(query_component_to_id)),
    //         6 => query = query.filter(schema::hd1_files_paths::p6.eq(query_component_to_id)),
    //         7 => query = query.filter(schema::hd1_files_paths::p7.eq(query_component_to_id)),
    //         8 => query = query.filter(schema::hd1_files_paths::p8.eq(query_component_to_id)),
    //         9 => query = query.filter(schema::hd1_files_paths::p9.eq(query_component_to_id)),
    //         10 => query = query.filter(schema::hd1_files_paths::p10.eq(query_component_to_id)),
    //         _ => return Err(StorDieselError::query_fail("path too big")),
    //     }
    // }
    // let query_as_component = schema::hd1_files_components::table.inner_join(query);
    // let children_ids = query.get_results::<u32>(conn.inner())?;

    // // todo: because small we can requery but if too big then building this becomes enormous
    // let component_to_id_vec = schema::hd1_files_components::table
    //     .filter(schema::hd1_files_components::id.eq_any(&children_ids))
    //     .get_results::<(u32, Vec<u8>)>(conn.inner())?;
    // let component_to_id = component_to_id_vec.into_iter().collect::<HashMap<_, _>>();
    //
    // let res = children_ids
    //     .iter()
    //     .map(|v| {
    //         let raw = component_to_id.get(v).unwrap();
    //         str::from_utf8(&raw).unwrap().to_string()
    //     })
    //     .collect();
    Ok(res)
}
