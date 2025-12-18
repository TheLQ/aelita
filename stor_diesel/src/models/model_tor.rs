use crate::models::diesel_wrappers::{TorHashV1Diesel, TorHashV2Diesel};
use crate::models::id_types::{ModelJournalId, ModelQbHostId, ModelTorrentState};
use chrono::NaiveDateTime;
use diesel::{HasQuery, Insertable};
use serde::{Deserialize, Serialize};
use xana_commons_rs::bencode_torrent_re::{TorHashV1, TorHashV2};

#[derive(HasQuery, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::tor1_torrents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelTorrents {
    pub journal_id: ModelJournalId,
    pub qb_host_id: ModelQbHostId,
    #[diesel(serialize_as = TorHashV1Diesel, deserialize_as = TorHashV1Diesel)]
    pub infohash_v1: TorHashV1,
    #[diesel(serialize_as = TorHashV2Diesel, deserialize_as = TorHashV2Diesel)]
    pub infohash_v2: TorHashV2,
    pub name: String,
    pub comment: String,
    pub path: String,
    pub progress: f32,
    pub original_size: u64,
    pub selected_size: u64,
    pub downloaded: u64,
    pub uploaded: u64,
    pub secs_active: u32,
    pub secs_seeding: u32,
    pub added_on: NaiveDateTime,
    pub completion_on: NaiveDateTime,
    pub state: ModelTorrentState,
}

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::tor1_qb_host)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelQbHost {
    pub qb_host_id: ModelQbHostId,
    pub name: String,
    pub address: String,
}

impl ModelQbHost {
    pub fn gui_name(&self) -> String {
        format!("{}({})", self.name, self.address)
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::tor1_qb_host)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelQbHosts {
    name: String,
    address: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema_temp::fast_tor_update)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSuperfast {
    pub tor_hash: TorHashV1Diesel,
    pub tor_state: String,
}
