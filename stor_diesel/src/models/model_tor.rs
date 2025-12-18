use crate::models::diesel_wrappers::{TorHashV1Diesel, TorHashV2Diesel};
use crate::models::id_types::{ModelJournalId, ModelQbHostId, ModelTorrentState};
use crate::{StorDieselError, StorDieselResult};
use chrono::{DateTime, NaiveDateTime};
use diesel::{HasQuery, Insertable, Queryable, QueryableByName};
use serde::{Deserialize, Serialize};
use xana_commons_rs::bencode_torrent_re::{
    ByTorHash, SHA1_BYTES, SHA256_BYTES, TorHashArray, TorHashV1, TorHashV2,
};
use xana_commons_rs::qbittorrent_re::TorrentState;

#[derive(HasQuery, Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::tor1_torrents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelTorrentsMeta {
    pub journal_id: ModelJournalId,
    pub qb_host_id: ModelQbHostId,
}

/// - integer dates
/// - empty v2 hash string
#[derive(Serialize, Deserialize)]
pub struct ModelTorrentsQBittorrent {
    pub infohash_v1: TorHashV1,
    pub infohash_v2: TorHashV2,
    pub name: String,
    pub comment: String,
    #[serde(rename = "content_path")]
    pub path: String,
    pub progress: f32,
    #[serde(default, rename = "total_size")]
    pub original_size: i64,
    #[serde(rename = "size")]
    pub selected_size: i64,
    pub downloaded: u64,
    pub uploaded: u64,
    #[serde(rename = "time_active")]
    pub secs_active: u32,
    #[serde(rename = "seeding_time")]
    pub secs_seeding: u32,
    pub added_on: i64,
    pub completion_on: i64,
    pub state: ModelTorrentState,
}

#[derive(HasQuery, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::tor1_torrents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelTorrentsDiesel {
    #[diesel(serialize_as = TorHashV1Diesel, deserialize_as = TorHashV1Diesel)]
    pub infohash_v1: TorHashV1,
    #[diesel(serialize_as = TorHashV2Diesel, deserialize_as = TorHashV2Diesel)]
    pub infohash_v2: TorHashV2,
    pub name: String,
    pub comment: String,
    pub path: String,
    pub progress: f32,
    pub original_size: Option<u64>,
    pub selected_size: Option<u64>,
    pub downloaded: u64,
    pub uploaded: u64,
    pub secs_active: u32,
    pub secs_seeding: u32,
    pub added_on: NaiveDateTime,
    pub completion_on: Option<NaiveDateTime>,
    pub state: ModelTorrentState,
}

impl TryFrom<ModelTorrentsQBittorrent> for ModelTorrentsDiesel {
    type Error = StorDieselError;

    fn try_from(
        ModelTorrentsQBittorrent {
            infohash_v1,
            infohash_v2,
            name,
            comment,
            path,
            progress,
            original_size,
            selected_size,
            downloaded,
            uploaded,
            secs_active,
            secs_seeding,
            added_on,
            completion_on,
            state,
        }: ModelTorrentsQBittorrent,
    ) -> StorDieselResult<Self> {
        Ok(Self {
            infohash_v1,
            infohash_v2,
            name,
            comment,
            path,
            progress,
            original_size: negative_one_to_none(original_size),
            selected_size: negative_one_to_none(selected_size),
            downloaded,
            uploaded,
            secs_active,
            secs_seeding,
            added_on: DateTime::from_timestamp_secs(added_on)
                .ok_or_else(|| StorDieselError::unknown_timestamp("valid added_on timestamp"))?
                .naive_utc(),
            completion_on: if let Some(timestamp) = negative_one_to_none(completion_on) {
                Some(
                    DateTime::from_timestamp_secs(timestamp as i64)
                        .ok_or_else(|| {
                            StorDieselError::unknown_timestamp("valid completion_on timestamp")
                        })?
                        .naive_utc(),
                )
            } else {
                None
            },
            state,
        })
    }
}

impl ByTorHash<SHA1_BYTES> for ModelTorrentsQBittorrent {
    fn tor_hash(&self) -> &TorHashArray<SHA1_BYTES> {
        &self.infohash_v1
    }
}

impl ByTorHash<SHA256_BYTES> for ModelTorrentsQBittorrent {
    fn tor_hash(&self) -> &TorHashArray<SHA256_BYTES> {
        &self.infohash_v2
    }
}

impl ByTorHash<SHA1_BYTES> for ModelTorrentsDiesel {
    fn tor_hash(&self) -> &TorHashArray<SHA1_BYTES> {
        &self.infohash_v1
    }
}

impl ByTorHash<SHA256_BYTES> for ModelTorrentsDiesel {
    fn tor_hash(&self) -> &TorHashArray<SHA256_BYTES> {
        &self.infohash_v2
    }
}

fn negative_one_to_none(value: i64) -> Option<u64> {
    match value {
        value if value >= 0 => Some(value as u64),
        -1 => None,
        negative => panic!("weird negative {negative}"),
    }
}

// impl From<ModelTorrentsWithIntegerDates> for ModelTorrents {}

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
