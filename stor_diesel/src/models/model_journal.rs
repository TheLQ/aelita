use crate::models::id_types::{ModelJournalId, ModelJournalType, ModelPublishId};
use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Debug)]
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

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::journal_data_immutable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalDataImmutable {
    pub publish_id: ModelPublishId,
    pub journal_id: ModelJournalId,
    pub journal_type: ModelJournalType,
    pub data: Vec<u8>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::journal_data_immutable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelJournalDataImmutable {
    pub publish_id: ModelPublishId,
    pub journal_type: ModelJournalType,
    pub data: Vec<u8>,
}
