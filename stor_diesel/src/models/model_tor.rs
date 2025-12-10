use crate::models::id_types::{ModelJournalId, ModelQbHostId, ModelTorrentState};
use crate::models::util_types::TorHashV1Diesel;
use diesel::{HasQuery, Insertable};

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::tor1_torrents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelTorrents {
    pub journal_id: ModelJournalId,
    pub torhash: TorHashV1Diesel,
    pub qb_host_id: ModelQbHostId,
    pub tor_status: ModelTorrentState,
    pub tor_status_changed: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::tor1_torrents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelTorrents {
    pub journal_id: ModelJournalId,
    pub torhash: TorHashV1Diesel,
    pub qb_host_id: ModelQbHostId,
    pub tor_status: ModelTorrentState,
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
