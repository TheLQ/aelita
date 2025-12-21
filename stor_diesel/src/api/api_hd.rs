use crate::api::common::SQL_PLACEHOLDER_MAX;
use crate::err::StorDieselErrorKind;
use crate::models::model_hd::HD_PATH_DEPTH;
use crate::{HdPathAssociation, NewHdPathAssociation, NewHdPathAssociationRoot, path_components};
use crate::{StorDieselResult, StorTransaction};
use crate::{schema, schema_temp};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use diesel::sql_types::Binary;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, LOCALE, SpaceJoiner};

pub fn storapi_hd_list_children(
    conn: &mut StorTransaction,
    path: impl AsRef<Path>,
) -> StorDieselResult<Vec<String>> {
    match 2 {
        1 => hd_list_children_paths(conn, path),
        2 => hd_list_children_parents(conn, path),
        _ => unimplemented!(),
    }
}

fn hd_list_children_parents(
    conn: &mut StorTransaction,
    path: impl AsRef<Path>,
) -> StorDieselResult<Vec<String>> {
    let path = path.as_ref();

    let path_components_bytes = path_components(path, |c| c.as_bytes())?;
    let path_components_str = path_components(path, |c| c.to_str().unwrap())?;
    let selected_column = path_components_bytes.len();

    let components_to_id_vec: Vec<(Vec<u8>, u32)> = schema::hd1_files_components::table
        .select((
            schema::hd1_files_components::component,
            schema::hd1_files_components::id,
        ))
        .filter(schema::hd1_files_components::component.eq_any(&path_components_bytes))
        .get_results(conn.inner())?;
    let components_to_id = components_to_id_vec
        .into_iter()
        .map(|(k, v)| (String::from_utf8(k).unwrap(), v))
        .collect::<HashMap<_, _>>();
    if components_to_id.len() != path_components_str.len() {
        let mut missing = path_components_str.clone();
        missing.retain(|v| !components_to_id.contains_key(*v));
        return Err(StorDieselErrorKind::UnknownComponent.build_message(format!(
            "{} total {}",
            missing.join(","),
            missing.len()
        )));
    }

    #[derive(QueryableByName)]
    struct PathResult {
        #[diesel(sql_type = diesel::sql_types::Text)]
        component: String,
    }

    let rows: Vec<PathResult>;
    if path_components_bytes.is_empty() {
        trace!("listing root");
        let query_builder = format!(
            //
            "SELECT DISTINCT \
            comp{selected_column}.component \
            FROM hd1_files_parents parents0 \
            INNER JOIN hd1_files_components comp{selected_column} ON comp{selected_column}.id = parents{selected_column}.component_id \
            WHERE \
                parents0.tree_depth = 0 AND \
                parents0.parent_id IS NULL"
        );
        rows = diesel::sql_query(query_builder).load::<PathResult>(conn.inner())?;
    } else {
        let prev_selected_column = selected_column - 1;
        trace!("listing {} components", path_components_bytes.len());
        let joins = (1..selected_column)
            .map(|i| {
                format!(
                    "INNER JOIN hd1_files_parents parents{i} ON \
                    parents{i}.tree_depth = {i} AND \
                    parents{i}.component_id = {comp_id} AND \
                    parents{i}.parent_id = parents{prev_i}.tree_id",
                    comp_id = components_to_id[path_components_str[i]],
                    prev_i = i - 1
                )
            })
            .collect::<SpaceJoiner>();
        let query_builder = format!(
            //
            "SELECT DISTINCT \
        comp.component \
        FROM hd1_files_parents parents0 \
        {joins} \
        INNER JOIN hd1_files_parents parents{selected_column} ON \
            parents{selected_column}.tree_depth = {selected_column} AND \
            parents{selected_column}.parent_id = parents{prev_selected_column}.tree_id \
        INNER JOIN hd1_files_components comp ON comp.id = parents{selected_column}.component_id \
        WHERE \
            parents0.tree_depth = 0 AND \
            parents0.component_id = {comp_id} AND \
            parents0.parent_id IS NULL \
        LIMIT 500",
            comp_id = components_to_id[path_components_str[0]]
        );
        rows = diesel::sql_query(query_builder).load::<PathResult>(conn.inner())?;
    }

    let res = rows.into_iter().map(|v| v.component).collect();
    Ok(res)
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

pub(super) fn components_get(
    conn: &mut StorTransaction,
    components_unique_input: &[&str],
) -> StorDieselResult<HashMap<String, u32>> {
    let watch = BasicWatch::start();
    let lookup_vec: Vec<(Vec<u8>, u32)> = schema_temp::fast_hd_components::table
        .inner_join(schema::hd1_files_components::table.on(
            schema::hd1_files_components::component.eq(schema_temp::fast_hd_components::component),
        ))
        .select((
            schema::hd1_files_components::component,
            schema::hd1_files_components::id,
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
    let associations = {
        let root_components = paths
            .iter()
            .map(|p| {
                let path_components = path_components(p.as_ref(), |o| o.to_str().unwrap());
                path_components
                    .map(|path_components| *component_to_id.get(path_components[0]).unwrap())
            })
            .try_collect::<_, Vec<_>, _>()?;
        let rows = diesel::insert_or_ignore_into(schema::hd1_files_parents::table)
            .values(
                root_components
                    .iter()
                    .map(|component_id| NewHdPathAssociationRoot {
                        component_id: *component_id,
                        tree_depth: 0,
                    })
                    .collect::<Vec<_>>(),
            )
            .execute(conn.inner())?;
        trace!("inserted {rows} root rows");

        HdPathAssociation::query()
            .filter(schema::hd1_files_parents::tree_depth.eq(0))
            .filter(schema::hd1_files_parents::component_id.eq_any(root_components))
            .get_results(conn.inner())?
    };
    let tree_id_to_associations = associations
        .into_iter()
        .map(|v| (v.tree_id, v))
        .collect::<HashMap<_, _>>();

    for comp_i in 0..HD_PATH_DEPTH {
        let parent_to_child_components: HashSet<(&str, &str)> = paths
            .iter()
            .filter_map(|v| {
                let path = match path_components(v.as_ref(), |o| o.to_str().unwrap()) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                let mut iter = path.iter().skip(comp_i);
                match (iter.next(), iter.next()) {
                    (None, _) => None,
                    (Some(_), None) => None,
                    (Some(parent), Some(child)) => Some(Ok((*parent, *child))),
                }
            })
            .try_collect::<_, HashSet<_>, _>()?;
    }

    let associations: HashSet<NewHdPathAssociation> = HashSet::new();
    // for path in paths {
    //     let path = path.as_ref();
    //     let mut path_iter = path.iter();
    //
    //     let mut prev = path_iter.next().unwrap().to_str().unwrap();
    //     while let Some(next_os) = path_iter.next() {
    //         let next = next_os.to_str().unwrap();
    //         associations.insert(NewHdPathAssociation {
    //             component_id: *component_to_id.get(next).unwrap(),
    //             parent_id: Some(*component_to_id.get(prev).unwrap()),
    //         });
    //         prev = next;
    //     }
    // }

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

#[cfg(test)]
mod test {
    use crate::storapi_hd_list_children;
    use crate::{PermaStore, StorDieselResult, StorTransaction, establish_connection};
    use aelita_commons::log_init;
    use xana_commons_rs::tracing_re::info;

    #[test]
    fn get_result() -> StorDieselResult<()> {
        sql_test(|conn| {
            let top_1 = "mnt";
            let top_2 = "hug24";
            let children = storapi_hd_list_children(conn, "/")?;
            info!("top {}", children.join(", "));
            let children = storapi_hd_list_children(conn, format!("/{top_1}"))?;
            info!("{top_1} {}", children.join(", "));
            let children = storapi_hd_list_children(conn, format!("/{top_2}"))?;
            info!("{top_2} {}", children.join(", "));
            panic!("??");
            Ok(())
        })
    }

    fn sql_test(
        inner: impl Fn(&mut StorTransaction) -> StorDieselResult<()>,
    ) -> StorDieselResult<()> {
        log_init();
        let conn = &mut establish_connection(PermaStore::AelitaNull).expect("bad conn");
        StorTransaction::new_transaction("test", conn, inner)?;
        Ok(())
    }
}
