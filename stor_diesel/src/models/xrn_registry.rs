use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::xrn_registry)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct XrnExtraction {
    pub xrn: String,
    pub published: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::xrn_registry)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewPost<'a> {
    pub xrn: &'a str,
    pub published: NaiveDateTime,
}