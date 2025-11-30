use crate::api::common::check_insert_num_rows;
use crate::connection::StorConnection;
use crate::err::StorDieselResult;
use crate::models::model_journal::{
    ModelJournalDataImmutable, ModelPublishLog, NewModelJournalDataImmutable,
    NewModelJournalDataImmutableDiesel, NewModelPublishLog,
};
use diesel::Connection;
use diesel::prelude::*;
use diesel::query_dsl::InternalJoinDsl;

pub fn storapi_journal_publish_push(
    conn: &mut StorConnection,
    input: NewModelPublishLog,
) -> StorDieselResult<()> {
    let result = diesel::insert_into(crate::schema::publish_log::table)
        .values(&input)
        .execute(conn);
    check_insert_num_rows(result, 1)
}

pub fn storapi_journal_publish_get(conn: &mut StorConnection) -> QueryResult<Vec<ModelPublishLog>> {
    crate::schema::publish_log::table.load::<ModelPublishLog>(conn)
}

pub fn storapi_journal_immutable_push(
    conn: &mut StorConnection,
    values_raw: impl IntoIterator<Item = NewModelJournalDataImmutable>,
) -> StorDieselResult<()> {
    conn.transaction(|conn| {
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
    })
}

pub fn storapi_journal_immutable_uncommitted(
    conn: &mut StorConnection,
) -> QueryResult<Vec<ModelJournalDataImmutable>> {
    conn.transaction(|conn| {
        crate::schema::journal_immutable::table
            .filter(crate::schema::journal_immutable::committed.eq(false))
            .load(conn)
    })
}
