use crate::models::date::StorDate;
use diesel::deserialize::FromSql;
use diesel::mysql::{Mysql, MysqlValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsChangeset, AsExpression, FromSqlRow, Insertable, Queryable, Selectable};
use std::io::Write;
use strum::{AsRefStr, EnumString};

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::jnl_mutation)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalMutation {
    pub(crate) mut_id: u32,
    pub(crate) mut_type: String,
    pub(crate) data: String,
    pub published: StorDate,
    pub publish_cause: String,
}

// #[derive(Selectable, QueryableByName, Insertable, Debug)]
#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::jnl_id_counters)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalIdCounter {
    pub key: ModelJournalIdKey,
    pub counter: u32,
    pub updated: StorDate,
}

#[derive(Debug, Clone, Copy, EnumString, AsRefStr, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub enum ModelJournalIdKey {
    Mutation,
    FireHistory,
}

impl FromSql<Text, Mysql> for ModelJournalIdKey {
    fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Mysql>>::from_sql(bytes)?;
        Ok(t.as_str().try_into()?)
    }
}

impl ToSql<Text, Mysql> for ModelJournalIdKey {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
        out.write(self.as_ref().as_bytes())?;
        Ok(IsNull::No)
    }
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = crate::schema::jnl_id_counters)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalIdCounterUpdate {
    pub counter: u32,
    pub updated: StorDate,
}
