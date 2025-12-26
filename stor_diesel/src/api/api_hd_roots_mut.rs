use crate::api::api_space_mut::storapi_space_new;
use crate::api::common::check_insert_num_rows;
use crate::models::model_hd_roots::{NewHdRoot, NewHdRootBuilder};
use crate::{ModelHdRoot, ModelJournalId, ModelSpaceId, NewModelSpaceName, schema};
use crate::{StorDieselError, StorDieselResult, StorTransaction};
use diesel::{ExpressionMethods, RunQueryDsl};

pub fn storapi_hdroots_push(
    conn: &mut StorTransaction,
    space: NewModelSpaceName,
    root: NewHdRoot,
) -> StorDieselResult<ModelSpaceId> {
    let space_id = storapi_space_new(conn, space)?;

    let rows = diesel::insert_into(schema::hd1_roots::table)
        .values((root, schema::hd1_roots::space_id.eq(space_id)))
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)?;

    Ok(space_id)
}

pub enum RootType {
    Primary,
    Backup,
}

// pub fn storapi_hdroots_set(conn: &mut StorTransaction, root_type: RootType)
