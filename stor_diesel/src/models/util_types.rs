use crate::StorDieselResult;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::internal::derives::multiconnection::array_comparison::In;
use diesel::mysql::{Mysql, MysqlValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::{Binary, Json};
use serde::Serialize;
use std::io::Write;
use xana_commons_rs::bencode_torrent_re::TorHashV1;

#[derive(Debug, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = diesel::sql_types::Binary)]
pub struct TorHashV1Diesel(TorHashV1);

impl TorHashV1Diesel {
    pub fn inner_hash(&self) -> &TorHashV1 {
        &self.0
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
#[derive(Debug, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Json)]
pub struct RawDieselJson(Vec<u8>);

impl RawDieselJson {
    pub fn serialize<V: Serialize>(value: V) -> StorDieselResult<Self> {
        Ok(Self(serde_json::to_vec(&value)?))
    }

    pub fn deserialize<'d, D: serde::Deserialize<'d>>(&'d self) -> StorDieselResult<D> {
        serde_json::from_slice(&self.0).map_err(Into::into)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl FromSql<Json, Mysql> for RawDieselJson {
    fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
        let inner = bytes.as_bytes().to_vec();
        Ok(Self(inner))
    }
}

impl ToSql<Json, Mysql> for RawDieselJson {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
        out.write_all(self.0.as_slice())?;
        Ok(IsNull::No)
    }
}
