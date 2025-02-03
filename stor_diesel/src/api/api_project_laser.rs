use crate::api::common::StorConnection;
use crate::err::StorDieselResult;
use crate::models::model_project_laser::{ModelProjectLaserSql, NewModelProjectLaserSql};
use crate::schema::*;
use diesel::dsl::max;
use diesel::prelude::*;
use diesel::{Connection, QueryDsl};

pub fn storapi_project_lasers_list(
    conn: &mut StorConnection,
) -> StorDieselResult<Vec<ModelProjectLaserSql>> {
    aproject_lasers::table
        .select(ModelProjectLaserSql::as_select())
        .load(conn)
        .map_err(Into::into)
}

pub fn storapi_project_lasers_push(
    conn: &mut StorConnection,
    new: Vec<NewModelProjectLaserSql>,
) -> StorDieselResult<Vec<NewModelProjectLaserSql>> {
    conn.transaction(|conn| {
        // current auto, if the table isn't empty
        let auto_id_start: Option<u32> = aproject_lasers::table
            .select(max(aproject_lasers::columns::xrn_laser_id))
            .first(conn)?;
        let old_total = match auto_id_start {
            Some(id) => id + 1,
            None => 0,
        };
        todo!();

        // check_insert_num_rows(
        //     insert_into(aproject_lasers::table)
        //         .values(new)
        //         .execute(conn),
        //     new.len(),
        // )?;
        //
        // Ok(Vec::new())
    })
}
