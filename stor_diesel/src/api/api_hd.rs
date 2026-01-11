use crate::api::common::{SQL_PLACEHOLDER_MAX, check_insert_num_rows, chunky_iter};
use crate::{ModelFileCompId, StorDieselResult, StorTransaction};
use crate::{schema, schema_temp};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use std::collections::{HashMap, HashSet};
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, LOCALE};
// pub fn storapi_hd_get_by_id(
//     conn: &mut StorTransaction,
//     id: ModelFileTreeId,
// ) -> StorDieselResult<HdPathAssociation> {
//     Ok(HdPathAssociation::query()
//         .filter(schema::hd1_files_parents::tree_id.eq(id))
//         .first(conn.inner())?)
// }

// pub fn components_get<T: AsRef<[u8]>>(
//     conn: &mut StorTransaction,
//     input: impl IntoIterator<Item = T>,
// ) -> StorDieselResult<Vec<(ModelFileCompId, Vec<u8>)>> {
//     let mut values: Vec<&[u8]> = Vec::new();
//     for item in input {
//         values.push(item.as_ref());
//     }
//     // let values = input.into_iter().map(|v| v.as_ref()).collect::<Vec<_>>();
//     components_get_bytes(conn, &values)
// }

pub fn components_get_bytes(
    conn: &mut StorTransaction,
    filter_components: &[&[u8]],
) -> StorDieselResult<Vec<(ModelFileCompId, Vec<u8>)>> {
    schema::hd1_files_components::table
        .select((
            schema::hd1_files_components::id,
            schema::hd1_files_components::component,
        ))
        .filter(schema::hd1_files_components::component.eq_any(filter_components))
        .get_results(conn.inner())
        .map_err(Into::into)
}

pub fn components_get_from_fast(
    conn: &mut StorTransaction,
    check_components_unique_input: &[impl AsRef<[u8]>],
) -> StorDieselResult<HashMap<Vec<u8>, ModelFileCompId>> {
    let watch = BasicWatch::start();
    let lookup_vec: Vec<(Vec<u8>, ModelFileCompId)> = schema_temp::fast_hd_components::table
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
        // .map(|(key, i)| (String::from_utf8(key).unwrap(), i))
        .collect::<HashMap<_, _>>();
    info!(
        "fetched {} rows in {watch}",
        lookup_map.len().to_formatted_string(&LOCALE)
    );
    assert_eq!(lookup_map.len(), check_components_unique_input.len());

    Ok(lookup_map)
}

pub fn storapi_hd_components_with(
    conn: &mut StorTransaction,
    input: &[&[u8]],
) -> StorDieselResult<HashMap<Vec<u8>, u32>> {
    let mut found = HashMap::new();
    for chunk in chunky_iter(SQL_PLACEHOLDER_MAX, "comp-with", input) {
        let rows = schema::hd1_files_components::table
            .select((
                schema::hd1_files_components::component,
                schema::hd1_files_components::id,
            ))
            .filter(schema::hd1_files_components::component.eq_any(chunk))
            .get_results(conn.inner())?;
        found.extend(rows)
    }
    Ok(found)
}
