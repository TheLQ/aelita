use crate::api::common::{
    SQL_MAX_PACKET_SIZE, assert_test_database, check_insert_num_rows, mysql_last_id,
};
use crate::connection::StorTransaction;
use crate::err::{StorDieselErrorKind, StorDieselResult};
use crate::models::id_types::{ModelJournalId, StorIdType};
use crate::models::model_journal::{
    ModelJournalImmutable, NewModelJournalImmutable, NewModelJournalImmutableDiesel,
};
use crate::{ModelJournalImmutableDiesel, RawDieselBytes, TorHashV2Diesel, schema};
use chrono::NaiveDateTime;
use diesel::dsl;
use diesel::prelude::*;
use sha2::{Digest, Sha256};
use xana_commons_rs::BasicWatch;
use xana_commons_rs::CrashErrKind;
use xana_commons_rs::bencode_torrent_re::TorHashV2;
use xana_commons_rs::tracing_re::{debug, info};

pub fn storapi_journal_immutable_push_single(
    conn: &mut StorTransaction,
    value_raw: NewModelJournalImmutable,
) -> StorDieselResult<ModelJournalId> {
    let NewModelJournalImmutable {
        journal_type,
        data,
        metadata,
        cause_description,
        cause_xrn,
    } = value_raw;
    let data_hash = TorHashV2::from_raw(Sha256::digest(data.as_inner()).into());
    let row = NewModelJournalImmutableDiesel {
        journal_type,
        metadata,
        committed: false,
        cause_description,
        cause_xrn,
        data_hash,
    };
    let row = diesel::insert_into(schema::journal_immutable::table)
        .values(row)
        .execute(conn.inner());
    check_insert_num_rows(row, 1)?;

    let journal_id = ModelJournalId::new(mysql_last_id(conn.inner())?);

    if data.0.len() > SQL_MAX_PACKET_SIZE {
        // insert multiple journal data
        let chunks = data.0.chunks(SQL_MAX_PACKET_SIZE);
        for data in chunks {
            insert_journal_data(conn, journal_id, data)?;
        }
    } else {
        insert_journal_data(conn, journal_id, data.as_inner())?;
    }

    Ok(journal_id)
}

fn insert_journal_data(
    conn: &mut StorTransaction,
    journal_id: ModelJournalId,
    data: &[u8],
) -> StorDieselResult<()> {
    let row = diesel::insert_into(schema::journal_immutable_data::table)
        .values((
            schema::journal_immutable_data::journal_id.eq(journal_id),
            schema::journal_immutable_data::data.eq(data),
        ))
        .execute(conn.inner());
    check_insert_num_rows(row, 1)
}

pub fn storapi_journal_list(
    conn: &mut StorTransaction,
) -> StorDieselResult<Vec<ModelJournalImmutableDiesel>> {
    ModelJournalImmutableDiesel::query()
        .get_results(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_journal_get_data(
    conn: &mut StorTransaction,
    journal_id: ModelJournalId,
) -> StorDieselResult<RawDieselBytes> {
    let hash: Option<TorHashV2Diesel> = schema::journal_immutable::table
        .select(schema::journal_immutable::data_hash)
        .filter(schema::journal_immutable::journal_id.eq(journal_id))
        .first(conn.inner())?;

    let datas: Vec<Vec<u8>> = schema::journal_immutable_data::table
        .select(schema::journal_immutable_data::data)
        .filter(schema::journal_immutable_data::journal_id.eq(journal_id))
        .order_by(schema::journal_immutable_data::data_id)
        .get_results(conn.inner())?;
    if datas.is_empty() {
        Err(StorDieselErrorKind::EmptyResult.build())
    } else {
        let mut datas = datas.into_iter();
        let mut total = datas.next().unwrap();
        for data in datas {
            total.extend(data)
        }

        if let Some(db_hash) = hash {
            let data_hash = Sha256::digest(&total);
            if data_hash.as_slice() == db_hash.inner_hash().to_raw() {
                debug!(
                    "checked {} bytes match hash {}",
                    total.len(),
                    db_hash.inner_hash()
                );
            } else {
                return Err(
                    StorDieselErrorKind::JournalHashFailed.build_message(format!(
                        "expected {} got {}",
                        db_hash.inner_hash(),
                        TorHashV2::from_raw(data_hash.into())
                    )),
                );
            }
        } else {
            debug!("No hash for journal {journal_id}");
        }

        Ok(RawDieselBytes(total))
    }
}

pub fn storapi_journal_get(
    conn: &mut StorTransaction,
    journal_id: ModelJournalId,
) -> StorDieselResult<ModelJournalImmutableDiesel> {
    ModelJournalImmutableDiesel::query()
        .filter(schema::journal_immutable::journal_id.eq(journal_id))
        .first(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_journal_get_created(
    conn: &mut StorTransaction,
    journal_id: ModelJournalId,
) -> StorDieselResult<NaiveDateTime> {
    schema::journal_immutable::table
        .select(schema::journal_immutable::at)
        .filter(schema::journal_immutable::journal_id.eq(journal_id))
        .first(conn.inner())
        .map_err(Into::into)
}

pub fn storapi_journal_commit_remain_next(
    conn: &mut StorTransaction,
) -> StorDieselResult<Option<ModelJournalImmutable>> {
    let watch = BasicWatch::start();

    let ModelJournalImmutableDiesel {
        journal_id,
        journal_type,
        at,
        metadata,
        committed,
        cause_description,
        cause_xrn,
        data_hash,
    } = match ModelJournalImmutableDiesel::query()
        .filter(schema::journal_immutable::committed.eq(false))
        .order_by(schema::journal_immutable::journal_id.asc())
        .first(conn.inner())
    {
        Ok(v) => v,
        Err(diesel::result::Error::NotFound) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let data = storapi_journal_get_data(conn, journal_id)?;

    debug!("Fetch journal {journal_id} in {watch}");
    Ok(Some(ModelJournalImmutable {
        journal_id,
        journal_type,
        at,
        data,
        metadata,
        committed,
        cause_description,
        cause_xrn,
        data_hash,
    }))
}

pub fn storapi_journal_commit_new(
    conn: &mut StorTransaction,
    to_commit: &ModelJournalId,
) -> StorDieselResult<()> {
    let expected_commit: Option<u32> = schema::journal_immutable::table
        .select(dsl::min(schema::journal_immutable::journal_id))
        .filter(schema::journal_immutable::committed.eq(false))
        .get_result(conn.inner())?;
    match expected_commit {
        Some(v) if v == to_commit.inner_id() => {
            // good
        }
        Some(v) => {
            return Err(StorDieselErrorKind::UnexpectedJournalIdForDatabase
                .build_message(format!("expected commit latest {v} but got {to_commit}")));
        }
        None => {
            return Err(StorDieselErrorKind::ZeroUncommittedJournals.build());
        }
    }

    let rows = diesel::update(schema::journal_immutable::table)
        .filter(schema::journal_immutable::journal_id.eq(to_commit))
        .set(schema::journal_immutable::committed.eq(true))
        .execute(conn.inner());
    check_insert_num_rows(rows, 1)
}

pub fn storapi_journal_uncommit_all(conn: &mut StorTransaction) -> StorDieselResult<()> {
    assert_test_database(conn)?;

    diesel::update(schema::journal_immutable::table)
        .set(schema::journal_immutable::committed.eq(false))
        .execute(conn.inner())?;

    Ok(())
}

pub fn storapi_reset_journal(conn: &mut StorTransaction) -> StorDieselResult<()> {
    assert_test_database(conn)?;

    if 1 + 1 == 2 {
        todo!()
    }
    let journal_rows = diesel::delete(schema::journal_immutable::table).execute(conn.inner())?;
    info!("Reset {journal_rows} journal rows");
    Ok(())
}
