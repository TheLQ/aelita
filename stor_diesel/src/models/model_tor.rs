use crate::models::id_types::{ModelJournalId, ModelQbHostId, ModelTorrentStatus};
use crate::models::util_types::TorHashV1Diesel;
use diesel::{HasQuery, Insertable};

#[derive(HasQuery, Insertable, Debug)]
#[diesel(table_name = crate::schema::tor1_torrents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelTorrents {
    journal_id: ModelJournalId,
    torhash: TorHashV1Diesel,
    qb_host_id: ModelQbHostId,
    tor_status_type: ModelTorrentStatus,
    tor_status_changed: chrono::NaiveDateTime,
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
