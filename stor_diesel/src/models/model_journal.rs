use crate::models::id_types::{ModelJournalId, ModelJournalTypeName, ModelPublishId};
use diesel::{HasQuery, Insertable};

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::publish_log)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelPublishLog {
    pub publish_id: ModelPublishId,
    pub at: chrono::NaiveDateTime,
    pub cause_xrn: Option<String>,
    pub cause_description: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::publish_log)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelPublishLog {
    pub cause_xrn: Option<String>,
    pub cause_description: String,
}

//

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::journal_immutable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalDataImmutable {
    pub publish_id: ModelPublishId,
    pub journal_id: ModelJournalId,
    pub journal_type: ModelJournalTypeName,
    pub data: Vec<u8>,
    pub committed: bool,
}

pub struct NewModelJournalDataImmutable {
    pub publish_id: ModelPublishId,
    pub journal_type: ModelJournalTypeName,
    pub data: Vec<u8>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::journal_immutable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelJournalDataImmutableDiesel {
    pub publish_id: ModelPublishId,
    pub journal_type: ModelJournalTypeName,
    pub data: Vec<u8>,
    pub committed: bool,
}
