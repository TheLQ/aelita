use crate::defs::address::{XrnAddr, XrnType};
use crate::defs::common::SubXrn;
use crate::err::LibxrnError;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::backtrace::Backtrace;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

/// xrn:project:paper/30305
/// xrn:project:task/3045
#[derive(Debug, Clone)]
pub struct SpaceXrn {
    ptype: ProjectTypeXrn,
    id: u32,
}

#[derive(Debug, Clone, AsRefStr, EnumString, PartialEq, Deserialize)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ProjectTypeXrn {
    Simple,
}

impl SpaceXrn {
    pub fn new(ptype: ProjectTypeXrn, id: u32) -> Self {
        Self { ptype, id }
    }

    pub fn ptype(&self) -> &ProjectTypeXrn {
        &self.ptype
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn into_addr(self) -> XrnAddr {
        self.into()
    }
}

impl SubXrn for SpaceXrn {
    fn atype() -> XrnType {
        XrnType::Project
    }
}

impl TryFrom<XrnAddr> for SpaceXrn {
    type Error = LibxrnError;

    fn try_from(addr: XrnAddr) -> Result<Self, Self::Error> {
        if addr.atype() != &XrnType::Project {
            return Err(LibxrnError::AddrInvalidType(addr, Backtrace::capture()));
        }

        let value = addr.value();
        let Some(sep) = value.find("/") else {
            return Err(LibxrnError::AddrMissingSeparator(
                addr,
                Backtrace::capture(),
            ));
        };
        let (ptype_raw, remain) = value.split_at(sep);
        let Ok(ptype) = ProjectTypeXrn::from_str(ptype_raw) else {
            return Err(LibxrnError::AddrInvalidType(addr, Backtrace::capture()));
        };

        let Ok(id) = remain[1..].parse() else {
            return Err(LibxrnError::AddrNotANumber(addr, Backtrace::capture()));
        };

        Ok(Self { ptype, id })
    }
}

impl From<SpaceXrn> for XrnAddr {
    fn from(SpaceXrn { ptype, id }: SpaceXrn) -> Self {
        XrnAddr::new(XrnType::Project, format!("{}/{}", ptype.as_ref(), id))
    }
}

impl FromStr for SpaceXrn {
    type Err = LibxrnError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr = XrnAddr::from_str(s)?;
        SpaceXrn::try_from(addr)
    }
}

impl<'de> Deserialize<'de> for SpaceXrn {
    fn deserialize<D>(deserializer: D) -> Result<SpaceXrn, D::Error>
    where
        D: Deserializer<'de>,
    {
        let addr = XrnAddr::deserialize(deserializer)?;
        SpaceXrn::try_from(addr).map_err(|e| D::Error::custom(format!("ProjectXrn_Serde {}", e)))
    }
}

impl Display for SpaceXrn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().into_addr())
    }
}

impl ProjectTypeXrn {
    pub fn into_xrn(self, id: u32) -> SpaceXrn {
        SpaceXrn::new(self, id)
    }
}

#[cfg(test)]
mod test {
    use crate::defs::space_xrn::{ProjectTypeXrn, SpaceXrn};

    #[test]
    fn convert_test() {
        let addr = SpaceXrn::new(ProjectTypeXrn::Simple, 123);
        assert_eq!(addr.into_addr().to_string(), "xrn:project:paper/123");
    }
}
