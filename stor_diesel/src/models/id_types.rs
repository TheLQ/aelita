use byteorder::NativeEndian;
use byteorder::WriteBytesExt;
use diesel::deserialize::FromSql;
use diesel::mysql::{Mysql, MysqlValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::{Integer, Unsigned};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name(u32);

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
    };
}
id_type!(ModelPublishId);
id_type!(ModelJournalId);

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
