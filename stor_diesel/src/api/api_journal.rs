use crate::api::common::check_insert_num_rows;
use crate::connection::StorConnection;
use crate::err::StorDieselResult;
use crate::models::date::StorDate;
use crate::models::model_journal::{
    ModelJournalIdCounter, ModelJournalIdCounterUpdate, ModelJournalIdKey, ModelJournalMutation,
};
use crate::schema::{jnl_id_counters, jnl_mutation};
use aelita_commons::tracing_re::debug;
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
        let existing_count = storapi_journal_id_counter_get_or_init(conn, ID_KEY_MUTATION)?;
        let mut existing_count_id = existing_count.counter;

        let va: Vec<ModelJournalMutation> = values
            .into_iter()
            .map(|NewMutation { mut_type, data }| {
                let res = ModelJournalMutation {
                    mut_id: existing_count_id,
                    mut_type,
                    data,
                    published: StorDate::now(),
                    publish_cause: "???".into(),
                };
                existing_count_id += 1;
                res
            })
            .collect();
        let va_len = va.len();
        debug!("len {}", va_len);

        let res = diesel::insert_into(jnl_mutation::table)
            .values(va)
            .execute(conn);
        check_insert_num_rows(res, va_len)?;

        storapi_journal_id_counter_update(conn, ID_KEY_MUTATION, existing_count_id)?;

        Ok(())
    })
}

pub fn storapi_journal_id_counter_get_or_init(
    conn: &mut StorConnection,
    key: ModelJournalIdKey,
) -> StorDieselResult<ModelJournalIdCounter> {
    let value = jnl_id_counters::table
        .find(&key)
        .select(ModelJournalIdCounter::as_select())
        .for_update() // row lock
        .first(conn)
        .optional()?;
    if let Some(value) = value {
        Ok(value)
    } else {
        let updated = StorDate::now();

        let res = diesel::insert_into(jnl_id_counters::table)
            .values(ModelJournalIdCounter {
                key,
                counter: 0,
                updated: updated.clone(),
            })
            .execute(conn);
        check_insert_num_rows(res, 1)?;

        Ok(ModelJournalIdCounter {
            key,
            counter: 0,
            updated,
        })
    }
}

pub fn storapi_journal_id_counter_update(
    conn: &mut StorConnection,
    key: ModelJournalIdKey,
    counter: u32,
) -> StorDieselResult<()> {
    let res = diesel::update(jnl_id_counters::table)
        .set(ModelJournalIdCounterUpdate {
            counter,
            updated: StorDate::now(),
        })
        .filter(jnl_id_counters::key.eq(key))
        .execute(conn);
    check_insert_num_rows(res, 1)?;
    Ok(())
}

pub fn storapi_journal_reset_all(conn: &mut StorConnection) -> StorDieselResult<()> {
    conn.transaction(|conn| {
        diesel::delete(jnl_mutation::table).execute(conn)?;
        diesel::delete(jnl_id_counters::table).execute(conn)?;
        Ok(())
    })
}
