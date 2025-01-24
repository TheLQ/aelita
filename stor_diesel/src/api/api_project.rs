use crate::api::common::{StorConnection, check_insert_num_rows};
use crate::err::StorDieselResult;
use crate::models::{ModelProject, ModelProjectSql, NewModelProject, NewModelProjectSql};
use crate::schema::{aproject_names, registry_ids};
use aelita_commons::tracing_re::{debug, trace};
use diesel::dsl::*;
use diesel::insert_into;
use diesel::prelude::*;
use std::ops::Range;

pub fn storapi_project_names_list(
    conn: &mut StorConnection,
) -> StorDieselResult<Vec<ModelProject>> {
    let projects_sql = aproject_names::table
        .select(ModelProjectSql::as_select())
        .load(conn)?;
    projects_sql
        .into_iter()
        .map(TryInto::try_into)
        .try_collect()
}

pub fn storapi_project_names_list_range(
    conn: &mut StorConnection,
    id_range: Range<u32>,
) -> StorDieselResult<Vec<ModelProject>> {
    debug!("range {}..{}", id_range.start, id_range.end);
    let projects_sql = aproject_names::table
        .select(ModelProjectSql::as_select())
        .filter(aproject_names::xrn_project_id.gt(id_range.start))
        .limit((id_range.end - id_range.start).into())
        .load(conn)?;
    projects_sql
        .into_iter()
        .map(TryInto::try_into)
        .try_collect()
}

pub fn storapi_project_names_push(
    conn: &mut StorConnection,
    new: Vec<NewModelProject>,
) -> StorDieselResult<Range<u32>> {
    let new: Vec<NewModelProjectSql> = new.into_iter().map(TryInto::try_into).try_collect()?;
    let new_len = new.len();

    let tx_res: StorDieselResult<Range<u32>> = conn.transaction(|conn| {
        // current auto, if the table isn't empty
        let auto_id_start: Option<u32> = aproject_names::table
            .select(max(aproject_names::columns::xrn_project_id))
            .first(conn)?;
        let old_total = auto_id_start.unwrap_or(0);

        check_insert_num_rows(
            new_len,
            insert_into(aproject_names::table).values(new).execute(conn),
        )?;

        let old_total = old_total as u32;
        let id_range = old_total..(old_total + new_len as u32);
        Ok(id_range)
    });
    Ok(tx_res?)
}

pub fn storapi_project_names_reset(conn: &mut StorConnection) -> StorDieselResult<usize> {
    Ok(delete(aproject_names::table).execute(conn)?)
}
