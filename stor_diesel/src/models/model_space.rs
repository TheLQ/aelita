use crate::models::id_types::{ModelPublishId, ModelSpaceId};
use diesel::{HasQuery, Insertable};

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::space_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSpaceNames {
    publish_id: ModelPublishId,
    space_id: ModelSpaceId,
    space_name: String,
    description: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::space_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelSpaceNames {
    pub publish_id: ModelPublishId,
    pub space_name: String,
    pub description: String,
}

#[derive(Insertable, HasQuery, Debug)]
#[diesel(table_name = crate::schema::space_owned)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSpaceOwned {
    pub publish_id: ModelPublishId,
    pub space_id: ModelSpaceId,
    pub child_xrn: String,
    pub description: String,
}
