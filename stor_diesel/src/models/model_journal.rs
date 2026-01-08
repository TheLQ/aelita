use crate::TorHashV2Diesel;
use crate::models::diesel_opt::OptTryInto;
use crate::models::diesel_wrappers::RawDieselBytes;
use crate::models::enum_types::ModelJournalTypeName;
use crate::models::id_types::ModelJournalId;
use chrono::NaiveDateTime;
use diesel::{HasQuery, Insertable};
use xana_commons_rs::bencode_torrent_re::TorHashV2;

pub struct ModelJournalImmutable {
    pub journal_id: ModelJournalId,
    pub journal_type: ModelJournalTypeName,
    pub at: NaiveDateTime,
    pub data: RawDieselBytes,
    pub metadata: Option<RawDieselBytes>,
    pub committed: bool,
    pub cause_description: String,
    pub cause_xrn: Option<String>,
    pub data_hash: Option<TorHashV2>,
}

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::journal_immutable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalImmutableDiesel {
    pub journal_id: ModelJournalId,
    pub journal_type: ModelJournalTypeName,
    pub at: NaiveDateTime,
    pub metadata: Option<RawDieselBytes>,
    pub committed: bool,
    pub cause_description: String,
    pub cause_xrn: Option<String>,
    #[diesel(serialize_as = OptTryInto<TorHashV2Diesel>, deserialize_as = OptTryInto<TorHashV2Diesel>)]
    pub data_hash: Option<TorHashV2>,
}

pub struct NewModelJournalImmutable {
    pub journal_type: ModelJournalTypeName,
    pub data: RawDieselBytes,
    pub metadata: Option<RawDieselBytes>,
    pub cause_description: String,
    pub cause_xrn: Option<String>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::journal_immutable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelJournalImmutableDiesel {
    pub journal_type: ModelJournalTypeName,
    pub metadata: Option<RawDieselBytes>,
    pub committed: bool,
    pub cause_description: String,
    pub cause_xrn: Option<String>,
    #[diesel(serialize_as = TorHashV2Diesel, deserialize_as = TorHashV2Diesel)]
    pub data_hash: TorHashV2,
}
