use crate::api::api_hd::storapi_hd_components_with;
use crate::err::StorDieselErrorKind;
use crate::{
    HdPathDieselDyn, ModelFileTreeId, PathRow, StorDieselResult, StorIdType, StorTransaction,
    components_get_bytes, path_components,
};
use aelita_xrn::defs::path_xrn::XRN_PATH_ROOT_ID;
use diesel::sql_types::Unsigned;
use diesel::{QueryableByName, RunQueryDsl};
use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{CommaJoiner, CrashErrKind, SpaceJoiner};

const LIMIT_CHILDREN_SIZE: u64 = 1000;

pub fn storapi_hd_get_path_by_id(
    conn: &mut StorTransaction,
    id: ModelFileTreeId,
) -> StorDieselResult<(Vec<PathRow>, PathBuf)> {
    info!("storapi_hd_get_path_by_id for {id}");
    if id.inner_id() == XRN_PATH_ROOT_ID {
        return Ok((Vec::new(), PathBuf::from("/")));
    }

    // todo can the second query be written to only select and join the last row?
    let raw_query = "\
        WITH RECURSIVE
        path_parts (tree_id, parent_id, component_id, tree_depth) AS (
            SELECT
                parents.tree_id,
                parents.parent_id,
                parents.component_id,
                parents.tree_depth
            FROM `hd1_files_parents` parents
            WHERE parents.tree_id = ?

            UNION ALL

            SELECT
                parents.tree_id,
                parents.parent_id,
                parents.component_id,
                parents.tree_depth
            FROM path_parts
            INNER JOIN `hd1_files_parents` parents ON
                parents.tree_id = path_parts.parent_id AND
                parents.tree_depth = path_parts.tree_depth - 1
            WHERE path_parts.tree_depth >= 0
        )
        SELECT path_parts.*, comp.component
        FROM path_parts
        INNER JOIN hd1_files_components comp on comp.id = path_parts.component_id
        ORDER BY path_parts.tree_depth ASC";
    let raw_query = raw_query.replace("\n", "");

    let rows: Vec<PathRow> = diesel::sql_query(raw_query)
        .bind::<Unsigned<diesel::sql_types::Integer>, _>(id)
        .get_results(conn.inner())?;
    let path: PathBuf = ["/"]
        .into_iter()
        .chain(rows.iter().map(|v| v.component.as_str()))
        .collect();

    Ok((rows, path))
}

pub fn storapi_hd_get_path_by_path(
    conn: &mut StorTransaction,
    path: impl AsRef<Path>,
) -> StorDieselResult<Vec<ModelFileTreeId>> {
    let path = path.as_ref();
    let components_str = path_components(path, |v| v.to_str().unwrap())?;
    info!(
        "Load components for path {}",
        components_str
            .iter()
            .map(|v| format!("\"{v}\""))
            .collect::<CommaJoiner>()
    );
    let components_len = components_str.len();
    if components_str.is_empty() {
        // we can't get the root component. all root's chuldren are null
        return Err(StorDieselErrorKind::EmptyPath.build());
    }

    let select_cols = (0..components_len)
        // .flat_map(|i| [format!("parents{i}.tree_id AS p{i}"), format!("parents{i}.tree_id AS c{i}")])
        .map(|i| format!("parents{i}.tree_id AS p{i}"))
        .collect::<CommaJoiner>();
    let joins = (1..components_len)
        .map(|i| {
            format!(
                "INNER JOIN hd1_files_parents parents{i} ON \
                    parents{i}.tree_depth = {i} AND \
                    parents{i}.component_id = ? AND \
                    parents{i}.parent_id = parents{prev_i}.tree_id ",
                prev_i = i - 1
            )
        })
        .collect::<SpaceJoiner>();
    let raw_query = format!(
        "SELECT {select_cols} \
        FROM `hd1_files_parents` parents0 \
        {joins} \
        WHERE \
            parents0.tree_depth = 0 AND \
            parents0.component_id = ? AND \
            parents0.parent_id IS NULL \
        "
    );

    let components_to_id = storapi_hd_components_with(conn, &components_str)?
        .into_iter()
        .collect::<HashMap<String, _>>();
    let mut query_components = components_str
        .iter()
        .map(|v| components_to_id.get(*v).unwrap())
        .collect::<Vec<_>>();

    let mut query = diesel::sql_query(raw_query).into_boxed();
    query_components.rotate_left(1); // first component in last where
    for i in query_components {
        query = query.bind::<Unsigned<diesel::sql_types::Integer>, _>(i)
    }
    let row = query.get_result::<HdPathDieselDyn>(conn.inner())?;
    Ok(row
        .components
        .into_iter()
        .map(ModelFileTreeId::new)
        .collect())
}

pub fn storapi_hd_list_children_by_id(
    conn: &mut StorTransaction,
    parent_id: ModelFileTreeId,
) -> StorDieselResult<Vec<PathRow>> {
    info!("storapi_hd_list_children_by_id for {parent_id}");

    // let mut query = schema::hd1_files_parents::table
    //     .inner_join(
    //         schema::hd1_files_components::table
    //             .on(schema::hd1_files_components::id.eq(schema::hd1_files_parents::component_id)),
    //     )
    //     .select((
    //         schema::hd1_files_parents::tree_id,
    //         schema::hd1_files_parents::tree_depth,
    //         schema::hd1_files_parents::component_id,
    //         schema::hd1_files_parents::parent_id,
    //         schema::hd1_files_components::component,
    //     ))
    //     .into_boxed();
    // if parent_id.inner_id() == XRN_PATH_ROOT_ID {
    //     query = query
    //         .filter(schema::hd1_files_parents::parent_id.is_null())
    //         .filter(schema::hd1_files_parents::tree_depth.eq(0))
    // } else {
    //     query = query.filter(schema::hd1_files_parents::parent_id.eq(parent_id))
    // }
    // todo wtf this doesn't work
    // .get_results::<PathRow>(conn.inner())
    // .map_err(Into::into)
    // let value = query.get_results::<(u32, u32, u32, Option<u32>, RawDieselBytes)>(conn.inner())?;
    // Ok(value
    //     .into_iter()
    //     .map(|v| PathRow {
    //         association: HdPathAssociation {
    //             tree_id: v.0,
    //             tree_depth: v.1,
    //             component_id: v.2,
    //             parent_id: v.3,
    //         },
    //         component: v.4.into(),
    //     })
    //     .collect())

    let raw_query = format!(
        "SELECT p.tree_id, p.tree_depth, p.component_id, p.parent_id, comp.component \
    FROM `hd1_files_parents` initial_p \
    INNER JOIN `hd1_files_parents` p ON
        p.parent_id = initial_p.tree_id AND
        p.tree_depth = initial_p.tree_depth + 1
    INNER JOIN `hd1_files_components` comp ON
        comp.id = p.component_id
    WHERE
        initial_p.tree_id = ?
    LIMIT {LIMIT_CHILDREN_SIZE}"
    );
    let query = diesel::sql_query(raw_query)
        .bind::<diesel::sql_types::Unsigned<diesel::sql_types::Integer>, _>(parent_id);

    query
        .get_results::<PathRow>(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_hd_list_children_by_path(
    conn: &mut StorTransaction,
    path: impl AsRef<Path>,
) -> StorDieselResult<Vec<String>> {
    let path = path.as_ref();

    let path_components_bytes = path_components(path, |c| c.as_bytes())?;
    let path_components_str = path_components(path, |c| c.to_str().unwrap())?;
    let selected_column = path_components_bytes.len();

    let components_to_id_vec = components_get_bytes(conn, &path_components_bytes)?;
    let components_to_id = components_to_id_vec
        .into_iter()
        .map(|(id, comp)| (String::from_utf8(comp).unwrap(), id))
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
        LIMIT {LIMIT_CHILDREN_SIZE}",
            comp_id = components_to_id[path_components_str[0]]
        );
        rows = diesel::sql_query(query_builder).load::<PathResult>(conn.inner())?;
    }

    let res = rows.into_iter().map(|v| v.component).collect();
    Ok(res)
}

#[cfg(test)]
mod test {
    use crate::api::common::test::sql_test;
    use crate::api::hd_path::tree_queries::{
        storapi_hd_get_path_by_path, storapi_hd_list_children_by_path,
    };
    use xana_commons_rs::PrettyUnwrap;
    use xana_commons_rs::tracing_re::info;

    #[test]
    fn get_result() {
        sql_test(|conn| {
            let top_1 = "mnt";
            let top_2 = "hug24";
            let children = storapi_hd_list_children_by_path(conn, "/")?;
            info!("top {}", children.join(", "));
            let children = storapi_hd_list_children_by_path(conn, format!("/{top_1}"))?;
            info!("{top_1} {}", children.join(", "));
            let children = storapi_hd_list_children_by_path(conn, format!("/{top_2}"))?;
            info!("{top_2} {}", children.join(", "));

            storapi_hd_get_path_by_path(conn, "").pretty_unwrap();

            panic!("??");
        })
        .pretty_unwrap()
    }
}
