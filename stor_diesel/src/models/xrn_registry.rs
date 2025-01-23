use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::xrn_registry)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct XrnExtraction {
    pub xrn: String,
    pub published: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::xrn_registry)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewXrnExtraction {
    pub xrn: String,
    pub published: String,
}
