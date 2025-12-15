use crate::models::id_types::{ModelJournalId, ModelSpaceId};
use diesel::{HasQuery, Insertable};

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::space_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSpaceName {
    pub journal_id: ModelJournalId,
    pub space_id: ModelSpaceId,
    pub space_name: String,
    pub description: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::space_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelSpaceName {
    pub journal_id: ModelJournalId,
    pub space_name: String,
    pub description: String,
}

#[derive(Insertable, HasQuery, Debug)]
#[diesel(table_name = crate::schema::space_owned)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSpaceOwned {
    pub journal_id: ModelJournalId,
    pub space_id: ModelSpaceId,
    pub child_xrn: String,
    pub description: String,
}
