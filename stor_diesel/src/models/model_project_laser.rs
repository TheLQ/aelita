use crate::models::date::StorDate;
use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::aproject_lasers)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelProjectLaserSql {
    xrn_laser_id: u32,
    title: String,
    description: String,
    published: StorDate,
    publish_cause: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::aproject_lasers)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelProjectLaserSql {
    pub title: String,
    pub description: String,
    pub published: StorDate,
    pub publish_cause: String,
}
