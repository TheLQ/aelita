use crate::StorDieselResult;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::mysql::{Mysql, MysqlValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::{Binary, Json};
use serde::Serialize;
use std::io::Write;
use xana_commons_rs::BasicWatch;
use xana_commons_rs::bencode_torrent_re::TorHashV1;
use xana_commons_rs::tracing_re::trace;

#[derive(Debug, diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Binary)]
pub struct TorHashV1Diesel(TorHashV1);

impl TorHashV1Diesel {
    pub fn inner_hash(&self) -> &TorHashV1 {
        &self.0
    }
}

impl From<TorHashV1> for TorHashV1Diesel {
    fn from(value: TorHashV1) -> Self {
        Self(value)
    }
}

impl<'t> From<&'t TorHashV1> for TorHashV1Diesel {
    fn from(value: &'t TorHashV1) -> Self {
        Self(value.clone())
    }
}

impl<Db: Backend> FromSql<Binary, Db> for TorHashV1Diesel
where
    *const [u8]: FromSql<Binary, Db>,
{
    fn from_sql(bytes: Db::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let inner = <Vec<u8> as FromSql<Binary, Db>>::from_sql(bytes)?;
        Ok(Self(TorHashV1::from_raw(inner.as_array().unwrap().clone())))
    }
}

impl<Db: Backend> ToSql<Binary, Db> for TorHashV1Diesel
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
    pub fn serialize_json<V: Serialize>(value: V) -> StorDieselResult<Self> {
        Ok(Self(serde_json::to_vec(&value)?))
    }

    pub fn deserialize_json<'d, D: serde::Deserialize<'d>>(&'d self) -> StorDieselResult<D> {
        serde_json::from_slice(&self.0).map_err(Into::into)
    }

    pub fn serialize_postcard<V: Serialize>(value: &V) -> StorDieselResult<Self> {
        Ok(Self(postcard::to_allocvec(value)?))
    }

    pub fn deserialize_postcard<'d, D: serde::Deserialize<'d>>(&'d self) -> StorDieselResult<D> {
        let watch = BasicWatch::start();
        let res = postcard::from_bytes(&self.0).map_err(Into::into);
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
