use crate::api::common::{StorConnection, check_insert_num_rows};
use crate::err::StorDieselResult;
use crate::models::{ModelProject, ModelProjectSql, NewModelProjectSql};
use crate::schema::aproject_names;
use diesel::insert_into;
use diesel::prelude::*;

pub fn storapi_project_names_list(
    connection: &mut StorConnection,
) -> StorDieselResult<Vec<ModelProject>> {
    let projects_sql = aproject_names::table
        .select(ModelProjectSql::as_select())
        .load(connection)?;
    projects_sql
        .into_iter()
        .map(TryInto::try_into)
        .try_collect()
}

pub fn storapi_project_names_push(
    connection: &mut StorConnection,
    new: Vec<NewModelProjectSql>,
) -> StorDieselResult<()> {
    check_insert_num_rows(
        new.len(),
        insert_into(aproject_names::table)
            .values(new)
            .execute(connection),
    )
}
