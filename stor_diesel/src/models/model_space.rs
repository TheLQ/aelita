use crate::StorDieselError;
use crate::models::common::parse_type_checked;
use crate::models::id_types::{ModelJournalId, ModelSpaceId};
use aelita_xrn::defs::address::XrnType;
use aelita_xrn::defs::path_xrn::{PathXrn, PathXrnType};
use aelita_xrn::defs::space_xrn::SpaceXrnType;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Binary;
use diesel::{HasQuery, Insertable, Selectable};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(HasQuery, Debug)]
#[diesel(table_name = crate::schema::space_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSpaceName {
    pub journal_id: ModelJournalId,
    pub space_id: ModelSpaceId,
    pub space_name: String,
    pub description: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::space_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelSpaceName {
    pub journal_id: ModelJournalId,
    pub space_name: String,
    pub description: String,
}

#[derive(Insertable, HasQuery, Debug)]
#[diesel(table_name = crate::schema::space_owned)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSpaceOwned {
    pub journal_id: ModelJournalId,
    pub space_id: ModelSpaceId,
    #[diesel(embed)]
    xrn: XrnDiesel,
    pub description: String,
}

/// Magic Xrn Parser struct
#[derive(Selectable, Debug)]
#[diesel(table_name = crate::schema::space_owned)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct XrnDiesel {
    child_type1: XrnType,
    child_type2: SumType<PathXrnType, SpaceXrnType>,
    child_id: u32,
}

impl TryFrom<&XrnDiesel> for PathXrn {
    type Error = &'static str;

    fn try_from(value: &XrnDiesel) -> Result<Self, &'static str> {
        if value.child_type1 != XrnType::Path {
            Err("child_type1 is not Path")
        } else if let Some(path_type) = &value.child_type2.left {
            Ok(PathXrn::new(*path_type, value.child_id, None))
        } else {
            Err("Unknown child_type2")
        }
    }
}

#[derive(Debug)]
pub struct SumType<Left, Right> {
    left: Option<Left>,
    right: Option<Right>,
}

impl<Left, Right> SumType<Left, Right> {
    fn new(left: Option<Left>, right: Option<Right>) -> Self {
        Self { left, right }
    }
}

impl<Left, Right, Db> FromSql<Binary, Db> for SumType<Left, Right>
where
    Db: Backend,
    Left: Debug + FromStr,
    Right: Debug + FromStr,
    <Left as FromStr>::Err: Display,
    <Right as FromStr>::Err: Display,
    *const [u8]: FromSql<Binary, Db>,
{
    fn from_sql(bytes: Db::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let raw_type = <Vec<u8> as FromSql<Binary, Db>>::from_sql(bytes)?;

        if let Ok(left) = parse_type_checked::<Left>(&raw_type) {
            Ok(Self {
                left: Some(left),
                right: None,
            })
        } else if let Ok(right) = parse_type_checked::<Right>(&raw_type) {
            Ok(Self {
                left: None,
                right: Some(right),
            })
        } else {
            let str_type = str::from_utf8(&raw_type).unwrap_or("UNKNOWN");
            Err(Box::new(StorDieselError::query_fail(format!(
                "Unknown type {str_type}",
            ))))
        }
    }
}

impl<Left, Right, Db> ToSql<Binary, Db> for SumType<Left, Right>
where
    Db: Backend,
    Left: Debug + AsRef<str>,
    Right: Debug + AsRef<str>,
    [u8]: ToSql<Binary, Db>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Db>) -> diesel::serialize::Result {
        let str_type = match (&self.left, &self.right) {
            (Some(left), None) => left.as_ref(),
            (None, Some(right)) => right.as_ref(),
            _ => panic!("SumType can only be one of two types"),
        };
        <[u8] as ToSql<Binary, Db>>::to_sql(str_type.as_bytes(), out)
    }
}
