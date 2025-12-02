use crate::models::id_types::{ModelJournalId, ModelJournalTypeName};
use diesel::{HasQuery, Insertable};

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::journal_immutable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalDataImmutable {
    pub journal_id: ModelJournalId,
    pub journal_type: ModelJournalTypeName,
    pub data: Vec<u8>,
    pub committed: bool,
    pub cause_description: String,
    pub cause_xrn: Option<String>,
}

pub struct NewModelJournalDataImmutable {
    pub journal_type: ModelJournalTypeName,
    pub data: Vec<u8>,
    pub cause_description: String,
    pub cause_xrn: Option<String>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::journal_immutable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelJournalDataImmutableDiesel {
    pub journal_type: ModelJournalTypeName,
    pub data: Vec<u8>,
    pub committed: bool,
    pub cause_description: String,
    pub cause_xrn: Option<String>,
}
