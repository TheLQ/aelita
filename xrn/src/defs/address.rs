/// xrn:project:1000000
#[derive(Debug)]
pub struct XrnAddr {
    atype: XrnAddrType,
    value: String,
}

use crate::err::LibxrnError;
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

#[derive(Debug, AsRefStr, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum XrnAddrType {
    /// A working project
    Project,
    /// A physically stored file
    A3,
    /// For displaying entities, this is a rating
    PlanningLabel,
    /// Syncs data from other sources to here
    SyncJob,
}

impl ToString for XrnAddr {
    fn to_string(&self) -> String {
        format!("xrn:{:?}:{}", self.atype, self.value)
    }
}

impl FromStr for XrnAddr {
    type Err = LibxrnError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 5 {
            return Err(LibxrnError::ParseShort(s.to_string()));
        }
        let (prefix, remain) = s.split_at(4);
        if prefix != "xrn:" {
            return Err(LibxrnError::ParsePrefix(s.to_string()));
        }

        let Some(next_sep) = remain.find(":") else {
            return Err(LibxrnError::ParsePrefixAfter(s.to_string()));
        };
        let (atype_raw, remain) = remain.split_at(next_sep);
        let atype = XrnAddrType::from_str(atype_raw)
            .map_err(|_| LibxrnError::InvalidType(atype_raw.to_string()))?;

        let (_ignore_comma, value) = remain.split_at(1);

        Ok(XrnAddr {
            atype,
            value: value.to_string(),
        })
    }
}
