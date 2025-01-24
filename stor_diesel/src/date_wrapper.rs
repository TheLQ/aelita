use crate::err::{StorDieselError, StorDieselResult};
use crate::models::date::StorDateType;
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[repr(transparent)]
pub struct StorDate(StorDateType);

impl StorDate {
    pub fn now() -> Self {
        Self(Local::now().into())
    }

    pub fn from_string(value: String) -> StorDieselResult<Self> {
        Self::from_str(&value)
    }

    pub fn to_stor_string(&self) -> String {
        self.0.to_rfc3339_opts(SecondsFormat::Secs, false)
    }
}

impl FromStr for StorDate {
    type Err = StorDieselError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(StorDateType::parse_from_rfc3339(s)?))
    }
}

impl Display for StorDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_stor_string())
    }
}

impl<'de> Deserialize<'de> for StorDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(StorDateType::deserialize(deserializer)?))
    }
}
