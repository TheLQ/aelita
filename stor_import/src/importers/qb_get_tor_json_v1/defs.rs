use crate::util::HashExtractor;
use aelita_stor_diesel::id_types::ModelQbHostId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use xana_commons_rs::bencode_torrent_re::TorHashV1;
use xana_commons_rs::qbittorrent_re::TorrentState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportQbMetadata {
    pub qb_host_id: ModelQbHostId,
}

#[derive(Debug, Deserialize)]
pub struct ImportQbTorrent {
    pub content_path: String,
    pub infohash_v1: TorHashV1,
    pub name: String,
    pub state: TorrentState,
    pub added_on: i64,
    pub completion_on: i64,
}

impl HashExtractor<ImportQbTorrent> for Vec<ImportQbTorrent> {
    fn as_tor_lookup_by_hash(&self) -> HashMap<&TorHashV1, &ImportQbTorrent> {
        self.iter().map(|v| (&v.infohash_v1, v)).collect()
    }
}
