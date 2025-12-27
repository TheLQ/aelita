use crate::api::common::SQL_PLACEHOLDER_MAX;
use crate::err::StorDieselErrorKind;
use crate::models::model_hd::HD_PATH_DEPTH;
use crate::{
    HdPathAssociation, ModelFileTreeId, ModelSpaceId, NewHdPathAssociation,
    NewHdPathAssociationRoot, PathRow, path_components,
};
use crate::{StorDieselResult, StorTransaction};
use crate::{schema, schema_temp};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use diesel::sql_types::{Binary, Unsigned};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};
use xana_commons_rs::CrashErrKind;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace};
use xana_commons_rs::{BasicWatch, LOCALE, SpaceJoiner};

// pub fn storapi_hd_get_by_id(
//     conn: &mut StorTransaction,
//     id: ModelFileTreeId,
// ) -> StorDieselResult<HdPathAssociation> {
//     Ok(HdPathAssociation::query()
//         .filter(schema::hd1_files_parents::tree_id.eq(id))
//         .first(conn.inner())?)
// }

pub(super) fn components_get(
    conn: &mut StorTransaction,
    components_unique_input: &[impl AsRef<str>],
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
