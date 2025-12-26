use crate::defs::address::{XrnAddr, XrnAddrRef, XrnMerge, XrnType};
use crate::defs::common::{SubXrnImpl, XrnSubTypeImpl, XrnTypeImpl, check_expected_type};
use crate::err::LibxrnError;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// xrn:project:paper/30305
/// xrn:project:task/3045
#[derive(Debug)]
pub struct SpaceXrn(XrnAddr);

impl SpaceXrn {
    pub fn new(stype: SpaceXrnType, id: u32, name: String) -> XrnAddr {
        XrnAddr(XrnMerge::Space(stype), id, name)
    }
}

impl XrnAddrRef for SpaceXrn {
    fn addr_ref(&self) -> &XrnAddr {
        &self.0
    }
}

impl SubXrnImpl for SpaceXrn {
    const UPPER: XrnType = XrnType::Space;
    type SubXrnType = SpaceXrnType;

    fn sub_type(&self) -> Self::SubXrnType {
        let XrnMerge::Space(kind) = self.addr_ref().merge() else {
            panic!("wut")
        };
        kind
    }
}

impl TryFrom<XrnAddr> for SpaceXrn {
    type Error = Box<LibxrnError>;
    fn try_from(addr: XrnAddr) -> Result<Self, Self::Error> {
        check_expected_type(Self::UPPER, &addr)?;
        Ok(Self(addr))
    }
}

impl FromStr for SpaceXrn {
    type Err = Box<LibxrnError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr = XrnAddr::from_str(s)?;
        addr.try_into()
    }
}

impl Display for SpaceXrn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        XrnAddr::fmt(&self.0, f)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    strum::AsRefStr,
    strum::EnumString,
    strum::VariantArray,
    strum::Display,
    //
)]
#[strum(serialize_all = "lowercase")]
// #[serde(rename_all = "lowercase")]
pub enum SpaceXrnType {
    Simple,
    RootPrimary,
    RootBackup,
}

impl XrnTypeImpl for SpaceXrnType {}

impl XrnSubTypeImpl for SpaceXrnType {}

#[cfg(test)]
mod test {
    use crate::defs::space_xrn::{SpaceXrn, SpaceXrnType};

    #[test]
    fn convert_test() {
        let addr = SpaceXrn::new(SpaceXrnType::Simple, 123, "asdf".to_string());
        assert_eq!(addr.to_string(), "xrn:space:simple:123:asdf");
    }
}
