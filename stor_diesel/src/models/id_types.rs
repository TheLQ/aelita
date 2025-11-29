use byteorder::NativeEndian;
use byteorder::WriteBytesExt;
use diesel::deserialize::FromSql;
use diesel::mysql::{Mysql, MysqlValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::{Integer, Text, Unsigned};
use diesel::{AsExpression, FromSqlRow};
use std::io::Write;
use strum::{AsRefStr, EnumString};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, AsExpression, diesel::FromSqlRow)]
        #[diesel(sql_type = Unsigned<Integer>)]
        pub struct $name(u32);

        // core conversions

        impl FromSql<Unsigned<Integer>, Mysql> for $name {
            fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
                let t = <u32 as FromSql<Unsigned<Integer>, Mysql>>::from_sql(bytes)?;
                Ok(Self(t))
            }
        }

        impl ToSql<Unsigned<Integer>, Mysql> for $name {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
                out.write_u32::<NativeEndian>(self.0)?;
                Ok(IsNull::No)
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
id_type!(ModelJournalType);

// #[derive(Debug, AsExpression, diesel::FromSqlRow)]
// // #[derive(Debug, diesel::FromSqlRow)]
// #[diesel(sql_type = Integer)]
// pub struct ModelPublishId(u32);
//
// impl diesel::expression::AsExpression<Unsigned<Integer>> for ModelPublishId {
//     type Expression = diesel::internal::derives::as_expression::Bound<Unsigned<Integer>, Self>;
//     fn as_expression(self) -> Self::Expression {
//         Self::Expression::new(self)
//     }
// }
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
// impl ToSql<Unsigned<Integer>, Mysql> for ModelPublishId {
//     fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
//         out.write_u32::<NativeEndian>(self.0)?;
//         Ok(IsNull::No)
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
