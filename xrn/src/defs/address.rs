use crate::defs::common::XrnTypeImpl;
use crate::err::{LibxrnError, XrnErrorKind};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantArray};

/// xrn:project:1000000
#[derive(Debug)]
pub struct XrnAddr {
    atype: XrnType,
    value: String,
}

impl XrnAddr {
    pub fn new(atype: XrnType, value: impl Into<String>) -> Self {
        Self {
            atype,
            value: value.into(),
        }
    }

    pub fn atype(&self) -> XrnType {
        self.atype
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Display for XrnAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "xrn:{}{}", self.atype.as_ref(), self.value)
    }
}

impl FromStr for XrnAddr {
    type Err = LibxrnError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let remain = match s.split_at_checked(3) {
            Some(("xrn", remain)) => remain,
            _ => return Err(XrnErrorKind::AddrPrefix.err_raw(s)),
        };

        let (atype, remain) = match XrnType::split_type(remain) {
            None => return Err(XrnErrorKind::AddrInvalidType.err_raw(s)),
            Some((_, "")) => return Err(XrnErrorKind::PathEmptyValue.err_raw(s)),
            Some(v) => v,
        };
        Ok(XrnAddr {
            atype,
            value: remain.to_string(),
        })
    }
}

#[derive(
    Clone, Copy, Debug, AsRefStr, EnumString, PartialEq, VariantArray, IntoStaticStr, Display,
)]
#[strum(serialize_all = "lowercase")]
pub enum XrnType {
    Project,
    Path,
}

impl XrnTypeImpl<'_> for XrnType {}

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
        deserializer.deserialize_string(XrnAddrVisitor)
    }
}

#[cfg(test)]
mod test {
    use crate::defs::address::{XrnAddr, XrnType};
    use crate::err::LibxrnResult;
    use std::str::FromStr;

    #[test]
    fn enum_test() {
        assert_eq!(XrnType::Project.as_ref(), "project");

        let addr = XrnAddr::new(XrnType::Project, ":page/123");
        assert_eq!(addr.to_string(), "xrn:project:page/123")
    }

    #[test]
    fn parse_test() -> LibxrnResult<()> {
        let addr_raw = "xrn:project:page/123";
        let addr = XrnAddr::from_str(addr_raw)?;
        assert_eq!(addr.atype(), XrnType::Project);
        assert_eq!(addr.value(), ":page/123");
        assert_eq!(addr.to_string(), addr_raw);

        let addr_raw = "xrn:path/page/123";
        let addr = XrnAddr::from_str(addr_raw)?;
        assert_eq!(addr.atype(), XrnType::Path);
        assert_eq!(addr.value(), "/page/123");
        assert_eq!(addr.to_string(), addr_raw);

        Ok(())
    }
}
