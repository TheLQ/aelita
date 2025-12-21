use crate::StorDieselError;
use crate::models::common::parse_type_checked;
use crate::models::enum_types::AnyEnumToText;
use crate::models::id_types::{ModelJournalId, ModelSpaceId};
use crate::schema;
use crate::schema::space_owned::dsl::space_owned;
use aelita_xrn::defs::address::{XrnAddr, XrnType};
use aelita_xrn::defs::path_xrn::{PathXrn, PathXrnType};
use aelita_xrn::defs::space_xrn::SpaceXrnType;
use diesel::backend::Backend;
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::query_builder::InsertStatement;
use diesel::row::NamedRow;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Binary, Nullable, Unsigned};
use diesel::{HasQuery, Insertable, QueryableByName, Selectable, Table};
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
    pub description: Option<String>,
    // #[diesel(embed)]
    // pub xrn: XrnDiesel,
}

#[derive(Insertable, HasQuery, diesel::QueryableByName, Debug)]
#[diesel(table_name = crate::schema::space_owned)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelSpaceXrn {
    pub child_type1: AnyEnumToText,
    pub child_type2: AnyEnumToText,
    pub child_id: u32,
}

/// Magic Xrn Parser struct
#[derive(Debug)]
pub struct XrnDiesel(PathXrn);

impl From<PathXrn> for XrnDiesel {
    fn from(value: PathXrn) -> Self {
        Self(value)
    }
}

impl From<XrnDiesel> for PathXrn {
    fn from(value: XrnDiesel) -> Self {
        value.0
    }
}

impl<DB: Backend> QueryableByName<DB> for XrnDiesel
where
    String: FromSql<diesel::sql_types::Text, DB>,
    u32: FromSql<Unsigned<diesel::sql_types::Integer>, DB>,
{
    fn build<'a>(row: &impl NamedRow<'a, DB>) -> diesel::deserialize::Result<Self> {
        let xrn_type_raw = NamedRow::get::<diesel::sql_types::Text, String>(row, "child_type1")?;
        let xrn_type = XrnType::from_str(&xrn_type_raw)?;
        if xrn_type != XrnType::Path {
            return Err(Box::new(StorDieselError::query_fail(format!(
                "Expected Path Xrn got {xrn_type}"
            ))));
        }

        let path_type_raw = NamedRow::get::<diesel::sql_types::Text, String>(row, "child_type2")?;
        let path_type = PathXrnType::from_str(&path_type_raw)?;

        let path_id = NamedRow::get::<Unsigned<diesel::sql_types::Integer>, u32>(row, "child_id")?;

        Ok(Self(PathXrn::new_id(path_type, path_id)))
    }
}

// impl TryFrom<&XrnDiesel> for PathXrn {
//     type Error = &'static str;
//
//     fn try_from(value: &XrnDiesel) -> Result<Self, &'static str> {
//         // if value.child_type1 != XrnType::Path {
//         //     Err("child_type1 is not Path")
//         // } else
//         if let Some(path_type) = &value.child_type2.left {
//             Ok(PathXrn::new_id(*path_type, value.child_id))
//         } else {
//             Err("Unknown child_type2")
//         }
//     }
// }

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
