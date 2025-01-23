use crate::defs::address::{XrnAddr, XrnAddrType};
use crate::err::LibxrnError;
use aelita_commons::err_utils::xbt;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

/// xrn:project:paper/30305
/// xrn:project:task/3045
#[derive(Debug, Clone)]
pub struct ProjectXrn {
    ptype: ProjectTypeXrn,
    id: u64,
}

#[derive(Debug, Clone, AsRefStr, EnumString, PartialEq, Deserialize)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ProjectTypeXrn {
    Paper,
    Task,
}

impl ProjectXrn {
    pub fn new(ptype: ProjectTypeXrn, id: u64) -> Self {
        Self { ptype, id }
    }

    pub fn ptype(&self) -> &ProjectTypeXrn {
        &self.ptype
    }

    pub fn id(&self) -> &u64 {
        &self.id
    }

    pub fn into_addr(self) -> XrnAddr {
        self.into()
    }
}

impl TryFrom<XrnAddr> for ProjectXrn {
    type Error = LibxrnError;

    fn try_from(addr: XrnAddr) -> Result<Self, Self::Error> {
        if addr.atype() != &XrnAddrType::Project {
            return Err(LibxrnError::AddrInvalidType(addr, xbt()));
        }

        let value = addr.value();
        let Some(sep) = value.find("/") else {
            return Err(LibxrnError::AddrMissingSeparator(addr));
        };
        let (ptype_raw, remain) = value.split_at(sep);
        let Ok(ptype) = ProjectTypeXrn::from_str(ptype_raw) else {
            return Err(LibxrnError::AddrInvalidType(addr, xbt()));
        };

        let Ok(id) = remain[1..].parse() else {
            return Err(LibxrnError::AddrNotANumber(addr));
        };

        Ok(Self { ptype, id })
    }
}

impl From<ProjectXrn> for XrnAddr {
    fn from(ProjectXrn { ptype, id }: ProjectXrn) -> Self {
        XrnAddr::new(XrnAddrType::Project, format!("{}/{}", ptype.as_ref(), id))
    }
}

impl FromStr for ProjectXrn {
    type Err = LibxrnError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr = XrnAddr::from_str(s)?;
        ProjectXrn::try_from(addr)
    }
}

impl<'de> Deserialize<'de> for ProjectXrn {
    fn deserialize<D>(deserializer: D) -> Result<ProjectXrn, D::Error>
    where
        D: Deserializer<'de>,
    {
        let addr = XrnAddr::deserialize(deserializer)?;
        ProjectXrn::try_from(addr).map_err(|e| D::Error::custom(format!("ProjectXrn_Serde {}", e)))
    }
}

impl Display for ProjectXrn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().into_addr())
    }
}

#[cfg(test)]
mod test {
    use crate::defs::project_xrn::{ProjectTypeXrn, ProjectXrn};

    #[test]
    fn convert_test() {
        let addr = ProjectXrn::new(ProjectTypeXrn::Paper, 123);
        assert_eq!(addr.into_addr().to_string(), "xrn:project:paper/123");
    }
}
