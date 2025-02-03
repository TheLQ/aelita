use crate::err::StorDieselError;
use crate::gen_try_from_converter;
use crate::models::date::StorDate;
use aelita_xrn::defs::address::XrnAddr;
use diesel::prelude::*;
use std::str::FromStr;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::registry_ids)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelRegistryId {
    pub xrn: String,
    pub published: StorDate,
    pub publish_cause: String,
}
