use crate::api::common::{StorConnection, check_insert_num_rows};
use crate::date_wrapper::StorDate;
use crate::err::StorDieselResult;
use crate::models::model_journal::{
    ModelJournalIdCounter, ModelJournalIdCounterUpdate, ModelJournalIdKey, ModelJournalMutation,
};
use crate::schema::jnl_id_counters::dsl::jnl_id_counters;
use crate::schema::jnl_mutation::dsl::jnl_mutation;
use diesel::Connection;
use diesel::prelude::*;

const ID_KEY_MUTATION: ModelJournalIdKey = ModelJournalIdKey::Mutation;
const MUT_TYPE: &str = "mut";

pub struct NewMutation {
    pub mut_type: String,
    pub data: String,
}

pub fn storapi_journal_mutation_push(
    conn: &mut StorConnection,
    values: impl IntoIterator<Item = NewMutation>,
) -> StorDieselResult<()> {
    conn.transaction(|conn| {
        let existing_count = match storapi_journal_id_counter_get_opt(conn, ID_KEY_MUTATION)? {
            Some(v) => v,
            None => {
                storapi_journal_id_counter_init(conn, ID_KEY_MUTATION)?;
                storapi_journal_id_counter_get_opt(conn, ID_KEY_MUTATION)?.unwrap()
            }
        };
        let mut existing_count_id = existing_count.counter;

        let va: Vec<ModelJournalMutation> = values
            .into_iter()
            .map(|NewMutation { mut_type, data }| {
                let res = ModelJournalMutation {
                    mut_id: existing_count_id,
                    mut_type: MUT_TYPE.into(),
                    data,
                    published: StorDate::now(),
                    publish_cause: "???".into(),
                };
                existing_count_id += 1;
                res
            })
            .collect();
        let va_len = va.len();

        let res = diesel::insert_into(jnl_mutation).values(va).execute(conn);
        check_insert_num_rows(res, va_len)?;

        Ok(())
    })
}

pub fn storapi_journal_id_counter_get_opt(
    conn: &mut StorConnection,
    key: ModelJournalIdKey,
) -> StorDieselResult<Option<ModelJournalIdCounter>> {
    jnl_id_counters
        .find(key.as_ref())
        .select(ModelJournalIdCounter::as_select())
        .for_update() // row lock
        .first(conn)
        .optional()
        .map_err(Into::into)
}

pub fn storapi_journal_id_counter_init(
    conn: &mut StorConnection,
    key: ModelJournalIdKey,
) -> StorDieselResult<()> {
    let res = diesel::insert_into(jnl_id_counters)
        .values(ModelJournalIdCounter {
            key,
            counter: 0,
            updated: StorDate::now(),
        })
        .execute(conn);
    check_insert_num_rows(res, 1)?;
    Ok(())
}

pub fn storapi_journal_id_counter_update(
    conn: &mut StorConnection,
    key: String,
    new: u32,
) -> StorDieselResult<()> {
    let res = diesel::insert_into(jnl_id_counters)
        .values(ModelJournalIdCounterUpdate {
            counter: 0,
            updated: StorDate::now(),
        })
        .execute(conn);
    check_insert_num_rows(res, 1)?;
    Ok(())
}
