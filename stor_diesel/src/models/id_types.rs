use aelita_xrn::defs::address::XrnAddrRef;
use aelita_xrn::defs::path_xrn::PathXrn;
use aelita_xrn::defs::space_xrn::SpaceXrn;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Integer, Unsigned};
use std::fmt::{Display, Formatter};

pub trait StorIdType {
    fn new(inner: u32) -> Self;

    fn inner_id(&self) -> u32;
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Copy,
            Clone,
            Debug,
            serde::Serialize,
            serde::Deserialize,
            diesel::expression::AsExpression,
            diesel::deserialize::FromSqlRow,
        )]
        // PartialEq,
        //             Eq,
        //             PartialOrd,
        //             Ord,
        #[diesel(sql_type = Unsigned<Integer>)]
        #[serde(transparent)]
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
id_type!(ModelJournalId);
id_type!(ModelSpaceId);
id_type!(ModelQbHostId);
id_type!(ModelFileTreeId);
id_type!(ModelFileCompId);

impl ModelSpaceId {
    pub fn from_project_xrn(xrn: &SpaceXrn) -> Self {
        Self(xrn.id())
    }
}

impl ModelFileTreeId {
    pub fn from_xrn(xrn: &PathXrn) -> Self {
        Self(xrn.id())
    }
}

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
