use aelita_stor_diesel::model_tor::ModelTorrentsQBittorrent;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use xana_commons_rs::bencode_torrent_re::{ByTorHash, SHA1_BYTES, TorHashArray, TorHashV1};

pub fn none_on_negative_deserializer<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = i64::deserialize(deserializer)?;
    if raw < 0 {
        Ok(None)
    } else {
        Ok(Some(raw as u64))
    }
}
