use crate::defs::address::{XrnAddr, XrnType};
use crate::defs::common::{SubXrn, XrnTypeImpl};
use crate::err::{LibxrnError, XrnErrorKind};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// xrn:project:paper/30305
/// xrn:project:task/3045
#[derive(Debug, Clone)]
pub struct SpaceXrn {
    stype: SpaceXrnType,
    id: u32,
    name: Option<String>,
}

impl SpaceXrn {
    pub fn new(stype: SpaceXrnType, id: u32, name: Option<String>) -> Self {
        Self { stype, id, name }
    }

    pub fn stype(&self) -> &SpaceXrnType {
        &self.stype
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl SubXrn for SpaceXrn {
    fn atype() -> XrnType {
        XrnType::Space
    }
}

impl From<&SpaceXrn> for XrnAddr {
    fn from(SpaceXrn { stype, id, name }: &SpaceXrn) -> Self {
        let mut value = format!("{stype}:{id}");
        if let Some(name) = name {
            value.push(':');
            value.push_str(&name);
        }
        XrnAddr::new(XrnType::Space, value)
    }
}

impl FromStr for SpaceXrn {
    type Err = Box<LibxrnError>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr = XrnAddr::from_str(s)?;
        SpaceXrn::try_from(addr)
    }
}

impl TryFrom<XrnAddr> for SpaceXrn {
    type Error = Box<LibxrnError>;

    fn try_from(addr: XrnAddr) -> Result<Self, Self::Error> {
        if addr.atype() != XrnType::Space {
            return Err(XrnErrorKind::SpaceInvalidInputType.build_message(addr));
        }

        let value = addr.value();
        let (stype, remain) = match SpaceXrnType::split_type(value) {
            None => return Err(XrnErrorKind::SpaceInvalidType.build_message(addr)),
            Some((_, "")) => return Err(XrnErrorKind::SpaceEmptyValue.build_message(addr)),
            Some(v) => v,
        };

        let next = remain.as_bytes().iter().skip(1).position(|v| v == &b':');
        let (id, remain) = match next.and_then(|v| value.split_at_checked(v)) {
            None => return Err(XrnErrorKind::InvalidSpaceId.build_message(addr)),
            Some((raw, remain)) => {
                let Ok(id) = raw.parse::<u32>() else {
                    return Err(XrnErrorKind::InvalidSpaceId.build_message(addr));
                };
                (id, remain)
            }
        };
        let name = if remain.is_empty() {
            None
        } else {
            Some(remain.to_string())
        };

        Ok(Self { stype, id, name })
    }
}

// impl<'de> Deserialize<'de> for SpaceXrn {
//     fn deserialize<D>(deserializer: D) -> Result<SpaceXrn, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let addr = XrnAddr::deserialize(deserializer)?;
//         SpaceXrn::try_from(addr).map_err(|e| D::Error::custom(format!("ProjectXrn_Serde {}", e)))
//     }
// }

impl Display for SpaceXrn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_addr())
    }
}

#[derive(
    Debug,
    Clone,
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
}

impl XrnTypeImpl<'_> for SpaceXrnType {}

#[cfg(test)]
mod test {
    use crate::defs::common::SubXrn;
    use crate::defs::space_xrn::{SpaceXrn, SpaceXrnType};

    #[test]
    fn convert_test() {
        let addr = SpaceXrn::new(SpaceXrnType::Simple, 123, None);
        assert_eq!(addr.to_addr().to_string(), "xrn:project:paper/123");
    }
}
