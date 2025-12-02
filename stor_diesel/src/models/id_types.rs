use crate::err::StorDieselError;
use crate::schema::sql_types::JournalImmutableJournalTypeEnum;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::mysql::{Mysql, MysqlValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::{Integer, Unsigned};
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::str::FromStr;

pub trait StorIdType {
    fn new(inner: u32) -> Self;

    fn inner_id(&self) -> u32;
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Copy, Clone, Debug, diesel::AsExpression, diesel::FromSqlRow)]
        // PartialEq,
        //             Eq,
        //             PartialOrd,
        //             Ord,
        #[diesel(sql_type = Unsigned<Integer>)]
        pub struct $name(u32);

        impl StorIdType for $name {
            fn new(inner: u32) -> Self {
                Self(inner)
            }

            fn inner_id(&self) -> u32 {
                self.0
            }
        }

        // core conversions

        impl<DB: Backend> FromSql<Unsigned<Integer>, DB> for $name
        where
            u32: FromSql<Unsigned<Integer>, DB>,
        {
            fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
                let inner = <u32 as FromSql<Unsigned<Integer>, DB>>::from_sql(bytes)?;
                Ok(Self(inner))
            }
        }

        impl<DB: Backend> ToSql<Unsigned<Integer>, DB> for $name
        where
            u32: ToSql<Unsigned<Integer>, DB>,
        {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> diesel::serialize::Result {
                <u32 as ToSql<Unsigned<Integer>, DB>>::to_sql(&self.0, out)
            }
        }

        // format macro

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        // seems unnessesary

        // impl From<u32> for $name {
        //     fn from(v: u32) -> Self {
        //         Self(v)
        //     }
        // }
        //

        // impl From<$name> for u32 {
        //     fn from(value: $name) -> u32 {
        //         value.0
        //     }
        // }
    };
}
id_type!(ModelPublishId);
id_type!(ModelJournalId);
id_type!(ModelSpaceId);

// #[derive(Debug, AsExpression, diesel::FromSqlRow)]
// #[diesel(sql_type = Unsigned<Integer>)]
// pub struct ModelPublishId(u32);
//
// impl From<ModelPublishId> for u32 {
//     fn from(value: ModelPublishId) -> u32 {
//         value.0
//     }
// }
//
// impl FromSql<Unsigned<Integer>, Mysql> for ModelPublishId {
//     fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
//         let t = <u32 as FromSql<Unsigned<Integer>, Mysql>>::from_sql(bytes)?;
//         Ok(Self(t))
//     }
// }
//
// impl<DB: Backend> ToSql<Unsigned<Integer>, DB> for ModelPublishId
// where
//     u32: ToSql<Unsigned<Integer>, DB>,
// {
//     fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> diesel::serialize::Result {
//         <u32 as ToSql<Unsigned<Integer>, DB>>::to_sql(&self.0, out)
//     }
// }

// #[derive(Debug)]
// pub struct TestId(u32);
//
// impl FromSql<Unsigned<Integer>, Mysql> for TestId {
//     fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
//         let t = <u32 as FromSql<Unsigned<Integer>, Mysql>>::from_sql(bytes)?;
//         Ok(Self(t))
//     }
// }
//
// impl ToSql<Unsigned<Integer>, Mysql> for TestId {
//     fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
//         out.write_u32::<NativeEndian>(self.0)?;
//         Ok(IsNull::No)
//     }
// }

macro_rules! enum_value {
    ($name:ident) => {
        /*
        todo: Assume we can use rust utf8 strings. Mysql only? Then why sqlite has a Binary+Text conversion?
        https://github.com/adwhit/diesel-derive-enum/blob/816ebe062a99056a69a194b4ba15532980558c19/src/lib.rs#L580
        */

        impl FromSql<JournalImmutableJournalTypeEnum, Mysql> for $name {
            fn from_sql(input_raw: MysqlValue) -> diesel::deserialize::Result<Self> {
                let input = str::from_utf8(input_raw.as_bytes()).map_err(|e| {
                    Box::new(StorDieselError::query_fail(format!(
                        "variant not bytes {e}"
                    )))
                })?;
                Self::from_str(input).map_err(|e| {
                    Box::new(StorDieselError::query_fail(format!(
                        "unsupported variant {e}"
                    )))
                    .into()
                })
            }
        }

        impl ToSql<JournalImmutableJournalTypeEnum, Mysql> for $name {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
                let as_str: &str = self.into();
                out.write_all(as_str.as_bytes())?;
                Ok(IsNull::No)
            }
        }
    };
}

#[derive(
    Debug,
    Hash,
    Eq,
    PartialEq,
    diesel::AsExpression,
    diesel::FromSqlRow,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[diesel(sql_type = JournalImmutableJournalTypeEnum)]
pub enum ModelJournalTypeName {
    Journal1,
    Space1,
}
enum_value!(ModelJournalTypeName);

// impl StorValues for ModelJournalTypeName {
//     type Id = ModelJournalTypeId;
//
//     fn to_int_id(&self) -> u32 {
//         match self {
//             Self::Journal1 => 1,
//             Self::Space1 => 2,
//         }
//     }
// }
