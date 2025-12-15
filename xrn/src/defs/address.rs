use crate::err::LibxrnError;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

/// xrn:project:1000000
#[derive(Debug)]
pub struct XrnAddr {
    atype: XrnType,
    value: String,
}

#[derive(Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum XrnType {
    Project,
}

impl XrnAddr {
    pub fn new(atype: XrnType, value: impl Into<String>) -> Self {
        Self {
            atype,
            value: value.into(),
        }
    }

    pub fn atype(&self) -> &XrnType {
        &self.atype
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Display for XrnAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "xrn:{}:{}", self.atype.as_ref(), self.value)
    }
}

impl FromStr for XrnAddr {
    type Err = LibxrnError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 5 {
            return Err(LibxrnError::ParseShort(s.into(), Backtrace::capture()));
        }
        let (prefix, remain) = s.split_at(4);
        if prefix != "xrn:" {
            return Err(LibxrnError::ParsePrefix(s.into(), Backtrace::capture()));
        }

        let Some(next_sep) = remain.find(":") else {
            return Err(LibxrnError::MissingSeparator(
                remain.into(),
                Backtrace::capture(),
            ));
        };
        let (atype_raw, remain) = remain.split_at(next_sep);
        let atype = XrnType::from_str(atype_raw)
            .map_err(|_| LibxrnError::InvalidType(atype_raw.into(), Backtrace::capture()))?;

        let (_ignore_comma, value) = remain.split_at(1);

        Ok(XrnAddr {
            atype,
            value: value.to_string(),
        })
    }
}

struct XrnAddrVisitor;

impl Visitor<'_> for XrnAddrVisitor {
    type Value = XrnAddr;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "xrn address")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        XrnAddr::from_str(v).map_err(|e| E::custom(format!("xrnaddr_serde {}", e)))
    }
}

impl<'de> Deserialize<'de> for XrnAddr {
    fn deserialize<D>(deserializer: D) -> Result<XrnAddr, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i32(XrnAddrVisitor)
    }
}

#[cfg(test)]
mod test {
    use crate::defs::address::{XrnAddr, XrnType};

    #[test]
    fn enum_test() {
        assert_eq!(XrnType::Project.as_ref(), "project");

        let addr = XrnAddr::new(XrnType::Project, "page/123");
        assert_eq!(addr.to_string(), "xrn:project:page/123")
    }
}
