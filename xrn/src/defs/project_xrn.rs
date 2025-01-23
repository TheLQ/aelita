use crate::defs::address::{XrnAddr, XrnAddrType};
use crate::err::{LibxrnError, LibxrnResult};
use aelita_commons::err_utils::xbt;
use std::cmp::PartialEq;
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

/// xrn:project:paper/30305
/// xrn:project:task/3045
#[derive(Debug)]
pub struct ProjectXrn {
    ptype: ProjectXrnType,
    id: usize,
}

#[derive(Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum ProjectXrnType {
    Paper,
    Task,
}

impl ProjectXrn {
    pub fn from_xrn(addr: XrnAddr) -> LibxrnResult<Self> {
        if addr.atype() != &XrnAddrType::Project {
            return Err(LibxrnError::AddrInvalidType(addr, xbt()));
        }

        let value = addr.value();
        let Some(sep) = value.find("/") else {
            return Err(LibxrnError::AddrMissingSeparator(addr));
        };
        let (ptype_raw, remain) = value.split_at(sep);
        let Ok(ptype) = ProjectXrnType::from_str(ptype_raw) else {
            return Err(LibxrnError::AddrInvalidType(addr, xbt()));
        };

        let Ok(id) = &remain[1..].parse() else {
            return Err(LibxrnError::AddrNotANumber(addr));
        };

        Ok(Self { ptype, id: *id })
    }
}

impl TryFrom<XrnAddr> for ProjectXrn {
    type Error = LibxrnError;

    fn try_from(value: XrnAddr) -> Result<Self, Self::Error> {
        Self::from_xrn(value)
    }
}
