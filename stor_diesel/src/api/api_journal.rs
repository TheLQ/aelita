use crate::api::common::{assert_test_database, check_insert_num_rows, mysql_last_id};
use crate::connection::{StorConnection, assert_in_transaction};
use crate::err::{StorDieselError, StorDieselResult};
use crate::models::id_types::{ModelJournalId, ModelPublishId, StorIdType};
use crate::models::model_journal::{
    ModelJournalDataImmutable, ModelPublishLog, NewModelJournalDataImmutable,
    NewModelJournalDataImmutableDiesel, NewModelPublishLog,
};
use diesel::dsl;
use diesel::prelude::*;
use xana_commons_rs::tracing_re::info;

pub fn storapi_journal_publish_push(
    conn: &mut StorConnection,
    input: NewModelPublishLog,
) -> StorDieselResult<ModelPublishId> {
    assert_in_transaction();

    let result = diesel::insert_into(crate::schema::publish_log::table)
        .values(&input)
        .execute(conn);
    check_insert_num_rows(result, 1)?;
    Ok(ModelPublishId::new(mysql_last_id(conn)?))
}

pub fn storapi_journal_publish_get(conn: &mut StorConnection) -> QueryResult<Vec<ModelPublishLog>> {
    assert_in_transaction();

    ModelPublishLog::query().load(conn)
}

pub fn storapi_journal_immutable_push(
    conn: &mut StorConnection,
    values_raw: impl IntoIterator<Item = NewModelJournalDataImmutable>,
) -> StorDieselResult<()> {
    assert_in_transaction();

    let values = values_raw
        .into_iter()
        .map(
            |NewModelJournalDataImmutable {
                 journal_type,
                 data,
                 publish_id,
             }| NewModelJournalDataImmutableDiesel {
                journal_type,
                data,
                publish_id,
                committed: false,
            },
        )
        .collect::<Vec<_>>();
    let values_len = values.len();
    let res = diesel::insert_into(crate::schema::journal_immutable::table)
        .values(&values)
        .execute(conn);
    check_insert_num_rows(res, values_len)
}

pub fn storapi_journal_commit_remain(
    conn: &mut StorConnection,
) -> QueryResult<Vec<ModelJournalDataImmutable>> {
    assert_in_transaction();

    ModelJournalDataImmutable::query()
        .filter(crate::schema::journal_immutable::committed.eq(false))
        .load(conn)
}

pub fn storapi_journal_commit_new(
    conn: &mut StorConnection,
    to_commit: ModelJournalId,
) -> StorDieselResult<()> {
    assert_in_transaction();

    let highest_committed: Option<u32> = crate::schema::journal_immutable::table
        .select(dsl::max(crate::schema::journal_immutable::journal_id))
        .filter(crate::schema::journal_immutable::committed.eq(true))
        .first(conn)?;
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
    let rows = diesel::update(crate::schema::journal_immutable::table)
        .filter(crate::schema::journal_immutable::journal_id.gt(to_commit))
        .set(crate::schema::journal_immutable::committed.eq(true))
        .execute(conn);
    check_insert_num_rows(rows, 1)
}

pub fn storapi_reset_journal(conn: &mut StorConnection) -> StorDieselResult<()> {
    assert_in_transaction();
    assert_test_database(conn)?;

    let journal_rows = diesel::delete(crate::schema::journal_immutable::table).execute(conn)?;
    let publish_rows = diesel::delete(crate::schema::publish_log::table).execute(conn)?;
    info!("Reset {journal_rows} journal {publish_rows} publish rows");
    Ok(())
}
