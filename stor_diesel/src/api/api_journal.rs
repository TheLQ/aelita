use crate::api::common::{assert_test_database, check_insert_num_rows};
use crate::connection::StorTransaction;
use crate::err::{StorDieselError, StorDieselResult};
use crate::models::id_types::{ModelJournalId, ModelSpaceId, StorIdType};
use crate::models::model_journal::{
    ModelJournalDataImmutable, NewModelJournalDataImmutable, NewModelJournalDataImmutableDiesel,
};
use crate::schema;
use diesel::dsl;
use diesel::prelude::*;
use xana_commons_rs::tracing_re::info;

pub fn storapi_journal_immutable_push_single(
    conn: &mut StorTransaction,
    value_raw: NewModelJournalDataImmutable,
) -> StorDieselResult<ModelJournalId> {
    let mut full = storapi_journal_immutable_push(conn, [value_raw])?;
    assert_eq!(full.len(), 1);
    Ok(full.remove(0))
}

pub fn storapi_journal_immutable_push(
    conn: &mut StorTransaction,
    values_raw: impl IntoIterator<Item = NewModelJournalDataImmutable>,
) -> StorDieselResult<Vec<ModelJournalId>> {
    let max_id: Option<ModelSpaceId> = schema::journal_immutable::table
        .select(dsl::max(schema::journal_immutable::journal_id))
        .get_result(conn.inner())?;

    let values = values_raw
        .into_iter()
        .map(
            |NewModelJournalDataImmutable {
                 journal_type,
                 data,
                 cause_xrn,
                 cause_description,
             }| NewModelJournalDataImmutableDiesel {
                journal_type,
                data,
                committed: false,
                cause_xrn,
                cause_description,
            },
        )
        .collect::<Vec<_>>();
    let values_len = values.len();
    let res = diesel::insert_into(schema::journal_immutable::table)
        .values(&values)
        .execute(conn.inner());
    check_insert_num_rows(res, values_len)?;

    if let Some(max_id) = max_id {
        let res = schema::journal_immutable::table
            .select(schema::journal_immutable::journal_id)
            .filter(schema::journal_immutable::journal_id.gt(max_id))
            .get_results(conn.inner())?;
        assert_eq!(res.len(), values_len);
        Ok(res)
    } else {
        schema::journal_immutable::table
            .select(schema::journal_immutable::journal_id)
            .get_results(conn.inner())
            .map_err(Into::into)
    }
}

pub fn storapi_journal_commit_remain(
    conn: &mut StorTransaction,
) -> StorDieselResult<Vec<ModelJournalDataImmutable>> {
    ModelJournalDataImmutable::query()
        .filter(schema::journal_immutable::committed.eq(false))
        .load(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_journal_commit_new(
    conn: &mut StorTransaction,
    to_commit: ModelJournalId,
) -> StorDieselResult<()> {
    let highest_committed: Option<u32> = schema::journal_immutable::table
        .select(dsl::max(schema::journal_immutable::journal_id))
        .filter(schema::journal_immutable::committed.eq(true))
        .first(conn.inner())?;
    if let Some(highest_committed) = highest_committed {
        if highest_committed + 1 != to_commit.inner_id() {
            return Err(StorDieselError::query_fail("cursor does not match"));
        }
    } else {
        // nothing commited yet
        if to_commit.inner_id() != 0 {
            return Err(StorDieselError::query_fail(
                "nothing committed, should be committing 0",
            ));
        }
    }
    let rows = diesel::update(schema::journal_immutable::table)
        .filter(schema::journal_immutable::journal_id.gt(to_commit))
        .set(schema::journal_immutable::committed.eq(true))
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)
}

pub fn storapi_reset_journal(conn: &mut StorTransaction) -> StorDieselResult<()> {
    assert_test_database(conn)?;

    let journal_rows = diesel::delete(schema::journal_immutable::table).execute(conn.inner())?;
    info!("Reset {journal_rows} journal rows");
    Ok(())
}
