use crate::models::id_types::{ModelPublishId, ModelSpaceId};
use diesel::{HasQuery, Insertable};

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::space)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSpace {
    publish_id: ModelPublishId,
    space_id: ModelSpaceId,
    space_name: String,
    description: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::space)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelSpace {
    publish_id: ModelPublishId,
    space_name: String,
    description: String,
}

#[derive(Insertable, HasQuery, Debug)]
#[diesel(table_name = crate::schema::space_owned)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSpaceOwned {
    publish_id: ModelPublishId,
    space_id: ModelSpaceId,
    child_xrn: String,
    description: String,
}
