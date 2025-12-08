use aelita_stor_diesel::id_types::ModelQbHostId;
use serde::{Deserialize, Serialize};
use xana_commons_rs::bencode_torrent_re::TorHashV1;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportQbMetadata {
    pub qb_host_id: ModelQbHostId,
}

#[derive(Debug, Deserialize)]
pub struct ImportQbTorrent {
    pub content_path: String,
    pub infhash_v1: TorHashV1,
}
