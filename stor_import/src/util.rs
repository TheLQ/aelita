use aelita_stor_diesel::model_tor::ModelTorrents;
use aelita_stor_diesel::util_types::TorHashV1Diesel;
use std::collections::HashMap;
use xana_commons_rs::bencode_torrent_re::TorHashV1;

pub trait HashExtractor<I> {
    fn as_tor_lookup_by_hash(&self) -> HashMap<&TorHashV1, &I>;
}

impl HashExtractor<ModelTorrents> for Vec<ModelTorrents> {
    fn as_tor_lookup_by_hash(&self) -> HashMap<&TorHashV1, &ModelTorrents> {
        self.iter().map(|v| (v.torhash.inner_hash(), v)).collect()
    }
}
