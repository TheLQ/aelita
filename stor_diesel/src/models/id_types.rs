use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::query_builder::bind_collector::RawBytesBindCollector;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Integer, Text, Unsigned};
use std::str::FromStr;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, diesel::AsExpression, diesel::FromSqlRow)]
        #[diesel(sql_type = Unsigned<Integer>)]
        pub struct $name(u32);

        impl $name {
            pub fn inner_id(&self) -> u32 {
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
        impl<DB: Backend> FromSql<Text, DB> for $name
        where
            String: FromSql<Text, DB>,
        {
            fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
                let inner = <String as FromSql<Text, DB>>::from_sql(bytes)?;
                Ok(Self::from_str(&inner)?)
            }
        }

        impl<DB> ToSql<Text, DB> for $name
        where
            // https://github.com/diesel-rs/diesel/blob/0abaf1b3f2ed24ac5643227baf841da9a63d9f1f/diesel/src/type_impls/primitives.rs#L143
            for<'a> DB: Backend<BindCollector<'a> = RawBytesBindCollector<DB>>,
        {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> diesel::serialize::Result {
                let inner: &'static str = self.into();
                <str as ToSql<Text, DB>>::to_sql(inner, out)
            }
        }
    };
}

#[derive(
    Debug, diesel::AsExpression, diesel::FromSqlRow, strum::EnumString, strum::IntoStaticStr,
)]
#[diesel(sql_type = Text)]
pub enum ModelJournalTypeName {
    Mutation,
    FireHistory,
}
enum_value!(ModelJournalTypeName);
