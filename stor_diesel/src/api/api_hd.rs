use crate::api::common::SQL_PLACEHOLDER_MAX;
use crate::api::fancy_chunk::{ChunkyAsRef, ChunkyPiece};
use crate::{ModelFileCompId, StorDieselResult, StorTransaction};
use crate::{schema, schema_temp};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use std::collections::HashMap;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{BasicWatch, LOCALE};

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
    input: &[impl AsRef<[u8]>],
) -> StorDieselResult<HashMap<Vec<u8>, u32>> {
    let mut found = HashMap::new();
    for chunk in ChunkyAsRef::new(input, "comp-with").pieces::<SQL_PLACEHOLDER_MAX>() {
        let rows = schema::hd1_files_components::table
            .select((
                schema::hd1_files_components::component,
                schema::hd1_files_components::id,
            ))
            .filter(schema::hd1_files_components::component.eq_any(chunk))
            .get_results(conn.inner())?;
        found.extend(rows)
    }
    if found.len() != input.len() {
        panic!(
            "not all components found, found {} input {}",
            found.len(),
            input.len()
        );
    }
    Ok(found)
}
