use crate::defs::address::{XrnAddr, XrnMerge, XrnType};
use crate::defs::common::XrnTypeImpl;
use crate::err::{LibxrnError, XrnErrorKind};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use xana_commons_rs::CrashErrKind;

/// xrn:project:paper/30305
/// xrn:project:task/3045
#[derive(Debug, Clone)]
pub struct SpaceXrn<'x>(&'x XrnAddr);

impl<'x> SpaceXrn<'x> {
    pub fn new(stype: SpaceXrnType, id: u32, name: String) -> XrnAddr {
        XrnAddr(XrnMerge::Space(stype), id, name)
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

#[cfg(test)]
mod test {
    use crate::defs::space_xrn::{SpaceXrn, SpaceXrnType};

    #[test]
    fn convert_test() {
        let addr = SpaceXrn::new(SpaceXrnType::Simple, 123, "asdf".to_string());
        assert_eq!(addr.to_string(), "xrn:space:simple:123:asdf");
    }
}
