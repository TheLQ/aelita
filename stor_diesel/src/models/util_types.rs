use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Binary;
use xana_commons_rs::bencode_torrent_re::TorHashV1;

#[derive(Debug, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Binary)]
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
        // todo: without clone
        let inner = <Vec<u8> as FromSql<Binary, Db>>::from_sql(bytes)?;
        Ok(Self(TorHashV1::from_raw(inner.as_array().unwrap().clone())))
    }
}

impl<Db: Backend> ToSql<Binary, Db> for TorHashV1Diesel
where
    [u8]: ToSql<Binary, Db>,
{
    // fn to_sql(bytes: Db::RawValue<'_>) -> diesel::deserialize::Result<Self> {
    //     <Vec<u8> as ToSql<Binary, Db>>::to_sql(bytes)?;
    //     Ok(Self(TorHashV1::from_raw(*inner.as_array().unwrap())))
    // }

    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Db>) -> diesel::serialize::Result {
        <[u8] as ToSql<Binary, Db>>::to_sql(self.0.to_raw(), out)
    }
}
