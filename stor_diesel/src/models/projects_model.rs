use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::xrn_registry)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelProject {
    pub xrn: String,
    pub published: NaiveDateTime,
}
