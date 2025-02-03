use crate::err::StorDieselError;
use crate::gen_try_from_converter;
use crate::models::date::StorDate;
use aelita_xrn::defs::address::XrnAddr;
use diesel::prelude::*;
use std::str::FromStr;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::registry_ids)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelRegistryIdSql {
    xrn: String,
    published: String,
    publish_cause: String,
}

#[derive(Debug)]
pub struct ModelRegistryId {
    pub xrn: XrnAddr,
    pub published: StorDate,
    pub publish_cause: String,
}

gen_try_from_converter!(
    ModelRegistryId,
    ModelRegistryIdSql,
    (publish_cause),
    (published, |v: StorDate| v.to_stor_string()),
    (xrn, |v: XrnAddr| v.to_string()),
);

gen_try_from_converter!(
    ModelRegistryIdSql,
    ModelRegistryId,
    (publish_cause),
    (published, StorDate::from_string),
    (xrn, |v: String| XrnAddr::from_str(&v)),
);
