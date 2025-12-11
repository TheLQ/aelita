use crate::err::StorDieselError;
use chrono::{DateTime, SecondsFormat, Utc};
use diesel::deserialize::FromSql;
use diesel::mysql::{Mysql, MysqlValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use std::str::FromStr;

pub type StorDateType = DateTime<Utc>;

#[derive(Clone, Default, diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)]
#[diesel(sql_type = Text)]
#[repr(transparent)]
pub struct StorDate(StorDateType);

impl StorDate {
    pub fn now() -> Self {
        Self(Utc::now())
    }
}

impl FromStr for StorDate {
    type Err = StorDieselError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl Display for StorDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = self.0.to_rfc3339_opts(SecondsFormat::Secs, true);
        f.write_str(&str)
    }
}

impl Debug for StorDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

//

impl FromSql<Text, Mysql> for StorDate {
    fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Mysql>>::from_sql(bytes)?;
        Ok(Self::from_str(&t)?)
    }
}

impl ToSql<Text, Mysql> for StorDate {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
        out.write(self.to_string().as_bytes())?;
        Ok(IsNull::No)
    }
}

// impl<'de> Deserialize<'de> for StorDate {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         Ok(Self(StorDateType::deserialize(deserializer)?))
//     }
// }
