use crate::err::{StorDieselError, StorDieselResult};
use crate::models::date::StorDateType;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Deserializer};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Default)]
#[repr(transparent)]
pub struct StorDate(StorDateType);

impl StorDate {
    pub fn now() -> Self {
        Self(Utc::now())
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
        Ok(Self(s.parse()?))
    }
}

impl TryFrom<String> for StorDate {
    type Error = StorDieselError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl Into<String> for StorDate {
    fn into(self) -> String {
        self.to_stor_string()
    }
}

impl Display for StorDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_stor_string())
    }
}

impl Debug for StorDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
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
