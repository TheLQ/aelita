use crate::{ModelHdRoot, ModelSpaceId};

#[derive(diesel::HasQuery, diesel::Insertable)]
#[diesel(table_name = crate::schema::hd1_roots)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct HdRoot {
    pub space_id: ModelSpaceId,
    pub rtype: ModelHdRoot,
}

#[derive(diesel::Insertable)]
#[diesel(table_name = crate::schema::hd1_roots)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewHdRoot {
    pub rtype: ModelHdRoot,
}
