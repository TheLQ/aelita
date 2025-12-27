use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::mysql::{Mysql, MysqlValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::{Binary, Json};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::mem::transmute;
use xana_commons_rs::BasicWatch;
use xana_commons_rs::bencode_torrent_re::{SHA1_BYTES, SHA256_BYTES, TorHashArray};
use xana_commons_rs::tracing_re::trace;

pub type TorHashV1Diesel = TorHashArrayDiesel<SHA1_BYTES>;
pub type TorHashV2Diesel = TorHashArrayDiesel<SHA256_BYTES>;

#[derive(
    Debug, diesel::expression::AsExpression, diesel::deserialize::FromSqlRow, Serialize, Deserialize,
)]
#[diesel(sql_type = diesel::sql_types::Binary)]
#[serde(transparent)]
#[repr(transparent)]
pub struct TorHashArrayDiesel<const SIZE: usize>(TorHashArray<SIZE>);

impl<const SIZE: usize> TorHashArrayDiesel<SIZE> {
    pub fn inner_hash(&self) -> &TorHashArray<SIZE> {
        &self.0
    }

    pub fn into_inner_hash(self) -> TorHashArray<SIZE> {
        self.0
    }
}

//

impl<const SIZE: usize> From<TorHashArray<SIZE>> for TorHashArrayDiesel<SIZE> {
    fn from(value: TorHashArray<SIZE>) -> Self {
        Self(value)
    }
}

impl<'t, const SIZE: usize> From<&'t TorHashArray<SIZE>> for TorHashArrayDiesel<SIZE> {
    fn from(value: &'t TorHashArray<SIZE>) -> Self {
        // cannot transmute ref to owned...
        Self(value.clone())
    }
}

impl<'t, const SIZE: usize> From<&'t TorHashArray<SIZE>> for &'t TorHashArrayDiesel<SIZE> {
    fn from(value: &'t TorHashArray<SIZE>) -> Self {
        unsafe { transmute(value) }
    }
}

//

impl<const SIZE: usize> From<TorHashArrayDiesel<SIZE>> for TorHashArray<SIZE> {
    fn from(value: TorHashArrayDiesel<SIZE>) -> Self {
        value.0
    }
}

impl<'t, const SIZE: usize> From<&'t TorHashArrayDiesel<SIZE>> for TorHashArray<SIZE> {
    fn from(value: &'t TorHashArrayDiesel<SIZE>) -> Self {
        value.0.clone()
    }
}

impl<'t, const SIZE: usize> From<&'t TorHashArrayDiesel<SIZE>> for &'t TorHashArray<SIZE> {
    fn from(value: &'t TorHashArrayDiesel<SIZE>) -> Self {
        unsafe { transmute(value) }
    }
}

//

impl<Db: Backend, const SIZE: usize> FromSql<Binary, Db> for TorHashArrayDiesel<SIZE>
where
    *const [u8]: FromSql<Binary, Db>,
{
    fn from_sql(bytes: Db::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let inner = <Vec<u8> as FromSql<Binary, Db>>::from_sql(bytes)?;
        Ok(Self(TorHashArray::from_raw(inner.try_into().unwrap())))
    }
}

impl<Db: Backend, const SIZE: usize> ToSql<Binary, Db> for TorHashArrayDiesel<SIZE>
where
    [u8]: ToSql<Binary, Db>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Db>) -> diesel::serialize::Result {
        <[u8] as ToSql<Binary, Db>>::to_sql(self.0.to_raw(), out)
    }
}

/// Raw JSON so the API isn't polluted with generics.
///
/// Further, FromSql/ToSql do not use generics because the
/// generic bounds get weird not actually being a BLOB type
#[derive(Debug, diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Binary)]
#[diesel(sql_type = Json)]
pub struct RawDieselBytes(pub Vec<u8>);

impl RawDieselBytes {
    pub fn serialize_json<V: Serialize>(value: V) -> serde_json::Result<Self> {
        Ok(Self(serde_json::to_vec(&value)?))
    }

    pub fn deserialize_json<'d, D: serde::Deserialize<'d>>(
        &'d self,
    ) -> Result<D, (String, serde_json::Error)> {
        serde_json::from_slice(&self.0).map_err(|e| {
            let len = self.0.len().min(10000);
            let extract = str::from_utf8(&self.0[0..len])
                .unwrap_or("[not a string]")
                .to_string();
            (extract, e)
        })
    }

    pub fn serialize_postcard<V: Serialize>(value: &V) -> postcard::Result<Self> {
        Ok(Self(postcard::to_allocvec(value)?))
    }

    pub fn deserialize_postcard<'d, D: serde::Deserialize<'d>>(&'d self) -> postcard::Result<D> {
        let watch = BasicWatch::start();
        let res = postcard::from_bytes(&self.0);
        trace!("Deserialized in {watch}");
        res
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn as_inner(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl FromSql<Json, Mysql> for RawDieselBytes {
    fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
        let inner = bytes.as_bytes().to_vec();
        Ok(Self(inner))
    }
}

impl ToSql<Json, Mysql> for RawDieselBytes {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
        out.write_all(self.0.as_slice())?;
        Ok(IsNull::No)
    }
}

impl<Db: Backend> FromSql<Binary, Db> for RawDieselBytes
where
    *const [u8]: FromSql<Binary, Db>,
{
    fn from_sql(bytes: Db::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let vec = <Vec<u8> as FromSql<Binary, Db>>::from_sql(bytes)?;
        Ok(Self(vec))
    }
}

impl<Db: Backend> ToSql<Binary, Db> for RawDieselBytes
where
    [u8]: ToSql<Binary, Db>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Db>) -> diesel::serialize::Result {
        <[u8] as ToSql<Binary, Db>>::to_sql(&self.0, out)
    }
}

// impl<DB> QueryableByName<DB> for XrnDiesel
// where
//     DB: Backend,
//     String: FromSql<diesel::sql_types::Text, DB>,
//     u32: FromSql<Unsigned<diesel::sql_types::Integer>, DB>,
// {
//     fn build<'a>(row: &impl NamedRow<'a, DB>) -> diesel::deserialize::Result<Self> {
//         let type1 = NamedRow::get::<diesel::sql_types::Text, _>(row, "child_type1")?;
//         let type2 = NamedRow::get::<diesel::sql_types::Text, _>(row, "child_type2")?;
//         let id = NamedRow::get::<Unsigned<diesel::sql_types::Integer>, _>(row, "id")?;
//         Ok(Self { type1, type2, id })
//     }
// }

// {
// use diesel;
// impl<__DB: diesel::backend::Backend> diesel::deserialize::QueryableByName<__DB> for PathResult
// where
//     String: diesel::deserialize::FromSql<diesel::sql_types::Text, __DB>,
// {
//     fn build<'__a>(row: &impl diesel::row::NamedRow<'__a, __DB>) -> diesel::deserialize::Result<Self> {
//         let mut component = {
//             let field = diesel::row::NamedRow::get::<diesel::sql_types::Text, String>(row, "component")?;
//             <String as std::convert::Into<String>>::into(field)
//         };
//         diesel::deserialize::Result::Ok(Self { component: component })
//     }
// }
// };
