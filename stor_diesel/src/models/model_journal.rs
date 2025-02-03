use crate::date_wrapper::StorDate;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::mysql::Mysql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, Associations, FromSqlRow, Insertable, Queryable, Selectable, SqlType};
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::jnl_mutation)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalMutation {
    pub(crate) mut_id: u32,
    pub(crate) mut_type: String,
    pub(crate) data: String,
    #[diesel(serialize_as = String)]
    pub published: StorDate,
    pub publish_cause: String,
}

// #[derive(Selectable, QueryableByName, Insertable, Debug)]
#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::jnl_id_counters)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalIdCounter {
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub key: ModelJournalIdKey,
    pub counter: u32,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub updated: StorDate,
}

// #[derive(Debug, EnumString, AsRefStr, FromSqlRow, AsExpression)]
#[derive(Debug, EnumString, AsRefStr, SqlType, FromSqlRow)]
#[diesel(sql_type = String)]
pub enum ModelJournalIdKey {
    Mutation,
}

// impl ToSql<Text, Mysql> for ModelJournalIdKey
// // where
// // DB: Backend,
// // String: ToSql<Text, DB>,
// {
//     fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
//         out.set_value(self.as_ref().to_string());
//         Ok(IsNull::No)
//         // let new = self.as_ref().to_string();
//         // new.to_sql(out)
//     }
// }
// impl FromSql<Text, Mysql> for ModelJournalIdKey
// // where
// //     DB: Backend,
// {
//     fn from_sql(bytes: <Mysql as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
//         let val: String = String::from_sql(bytes)?;
//         Self::from_str(&val).map_err(|e| format!("{}", e).into())
//     }
// }

impl TryFrom<String> for ModelJournalIdKey {
    type Error = strum::ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl Into<String> for ModelJournalIdKey {
    fn into(self) -> String {
        self.as_ref().to_string()
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::jnl_id_counters)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelJournalIdCounterUpdate {
    pub counter: u32,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub updated: StorDate,
}
