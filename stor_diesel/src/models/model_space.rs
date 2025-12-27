use crate::models::enum_types::AnyEnumToText;
use crate::models::id_types::{ModelJournalId, ModelSpaceId};
use aelita_xrn::defs::address::{XrnAddr, XrnAddrRef};
use diesel::{HasQuery, Insertable};
use std::fmt::Debug;

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
pub struct XrnAsOwnedTable {
    pub child_type1: AnyEnumToText,
    pub child_type2: AnyEnumToText,
    pub child_id: u32,
}

impl<X: Into<XrnAddr>> From<X> for XrnAsOwnedTable {
    fn from(value: X) -> Self {
        let value = value.into();
        let child_id = value.id();
        let merge = value.merge();
        let (upper, lower) = merge.types_as_str();
        Self {
            child_type1: AnyEnumToText::new(upper),
            child_type2: AnyEnumToText::new(lower),
            child_id,
        }
    }
}

// Magic Xrn Parser struct
// #[derive(Debug)]
// pub struct XrnDiesel(PathXrn);
//
// impl From<PathXrn> for XrnDiesel {
//     fn from(value: PathXrn) -> Self {
//         Self(value)
//     }
// }
//
// impl From<XrnDiesel> for PathXrn {
//     fn from(value: XrnDiesel) -> Self {
//         value.0
//     }
// }
//
// impl<DB: Backend> QueryableByName<DB> for XrnDiesel
// where
//     String: FromSql<diesel::sql_types::Text, DB>,
//     u32: FromSql<Unsigned<diesel::sql_types::Integer>, DB>,
// {
//     fn build<'a>(row: &impl NamedRow<'a, DB>) -> diesel::deserialize::Result<Self> {
//         let xrn_type_raw = NamedRow::get::<diesel::sql_types::Text, String>(row, "child_type1")?;
//         let xrn_type = XrnType::from_str(&xrn_type_raw)?;
//         if xrn_type != XrnType::Path {
//             return Err(StorDieselErrorKind::NotPathXrn.build_message(xrn_type));
//         }
//
//         let path_type_raw = NamedRow::get::<diesel::sql_types::Text, String>(row, "child_type2")?;
//         let path_type = PathXrnType::from_str(&path_type_raw)?;
//
//         let path_id = NamedRow::get::<Unsigned<diesel::sql_types::Integer>, u32>(row, "child_id")?;
//
//         Ok(Self(PathXrn::new_id(path_type, path_id)))
//     }
// }

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
