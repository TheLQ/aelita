use crate::date_wrapper::StorDate;
use crate::err::StorDieselError;
use crate::gen_try_from_converter;
use aelita_xrn::defs::address::XrnAddr;
use diesel::prelude::*;
use std::str::FromStr;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::registry_ids)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelRegistryIdSql {
    xrn: String,
    published: String,
}

#[derive(Debug)]
pub struct ModelRegistryId {
    pub xrn: XrnAddr,
    pub published: StorDate,
}

gen_try_from_converter!(
    ModelRegistryId,
    ModelRegistryIdSql,
    (),
    (published, |v: StorDate| v.to_stor_string()),
    (xrn, |v: XrnAddr| v.to_string()),
);

gen_try_from_converter!(
    ModelRegistryIdSql,
    ModelRegistryId,
    (),
    (published, StorDate::from_string),
    (xrn, |v: String| XrnAddr::from_str(&v)),
);

#[derive(Insertable)]
#[diesel(table_name = crate::schema::registry_ids)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelRegistryIdSql {
    xrn: String,
    published: String,
    publish_cause: String,
}

pub struct NewModelRegistryId {
    pub xrn: XrnAddr,
    pub published: StorDate,
    pub publish_cause: String,
}

gen_try_from_converter!(
    NewModelRegistryId,
    NewModelRegistryIdSql,
    (publish_cause),
    (published, |v: StorDate| v.to_stor_string()),
    (xrn, |v: XrnAddr| v.to_string()),
);

gen_try_from_converter!(
    NewModelRegistryIdSql,
    NewModelRegistryId,
    (publish_cause),
    (published, StorDate::from_string),
    (xrn, |v: String| XrnAddr::from_str(&v)),
);
