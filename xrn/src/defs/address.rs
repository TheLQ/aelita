use crate::defs::common::XrnTypeImpl;
use crate::defs::path_xrn::{PathXrnType, TREE_PREFIX_STR, XRN_PATH_ROOT_ID};
use crate::defs::space_xrn::SpaceXrnType;
use crate::err::{LibxrnError, XrnErrorKind};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;
use xana_commons_rs::CrashErrKind;

/// xrn:project:1000000
#[derive(Debug, Clone)]
pub struct XrnAddr(pub XrnMerge, pub u32, pub String);

impl XrnAddr {
    pub fn new(atype: XrnMerge, id: u32, value: impl Into<String>) -> Self {
        Self(atype, id, value.into())
    }
}

pub trait XrnAddrRef: FromStr<Err = Box<LibxrnError>> {
    fn addr_ref(&self) -> &XrnAddr;

    fn merge(&self) -> XrnMerge {
        self.addr_ref().0
    }

    fn id(&self) -> u32 {
        self.addr_ref().1
    }

    fn value(&self) -> &str {
        &self.addr_ref().2
    }
}

impl XrnAddrRef for XrnAddr {
    fn addr_ref(&self) -> &XrnAddr {
        self
    }
}

impl Display for XrnAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut standard_format = |upper, lower| {
            write!(f, "xrn:{}:{}:{}", upper, lower, self.id())?;
            let value = self.value();
            if !value.is_empty() {
                write!(f, ":{value}")?;
            }
            Ok(())
        };
        match self.merge() {
            upper @ XrnMerge::Path(lower) => {
                write!(
                    f,
                    "xrn:{}:{}{}{TREE_PREFIX_STR}{}",
                    upper.as_ref(),
                    lower.as_ref(),
                    self.value(),
                    self.id()
                )
            }
            upper @ XrnMerge::Space(lower) => standard_format(upper.as_ref(), lower.as_ref()),
        }
    }
}

impl FromStr for XrnAddr {
    type Err = Box<LibxrnError>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let remain = match s.split_at_checked(3) {
            Some(("xrn", remain)) => remain,
            _ => return Err(XrnErrorKind::InvalidPrefix.build_message(s)),
        };

        let (upper, remain) = match XrnType::split_type(remain) {
            None => return Err(XrnErrorKind::InvalidUpper.build_message(s)),
            Some(v) => v,
        };
        match upper {
            XrnType::Space => match SpaceXrnType::split_type(remain) {
                None => Err(XrnErrorKind::AddrInvalidType.build_message(s)),
                Some((v, remain)) => {
                    let (sep, remain) = match remain.split_at_checked(1) {
                        None => return Err(XrnErrorKind::AddrMissingPreIdSep.build_message(s)),
                        Some(v) => v,
                    };
                    if sep != ":" {
                        return Err(XrnErrorKind::AddrInvalidPreIdSep.build_message(s));
                    }

                    let Some(id_end) = remain.as_bytes().iter().position(|v| *v == b':') else {
                        return Err(XrnErrorKind::AddrMissingPreValueSep.build_message(s));
                    };
                    let (id_raw, remain) = remain.split_at(id_end);
                    let id = id_raw.parse::<u32>().map_err(
                        XrnErrorKind::AddrIdNotANumber
                            .err_message_fn_map(|| format!("input '{id_raw}'")),
                    )?;

                    Ok(Self(XrnMerge::Space(v), id, remain[1/*sep*/..].to_string()))
                }
            },
            XrnType::Path => match PathXrnType::split_type(remain) {
                None => Err(XrnErrorKind::PathInvalidType.build_message(s)),
                Some((xtype @ PathXrnType::Fs, path @ "/")) => Ok(Self(
                    XrnMerge::Path(xtype),
                    XRN_PATH_ROOT_ID,
                    path.to_string(),
                )),
                Some((v, remain)) => {
                    let Some(id_pos) = remain.rfind(TREE_PREFIX_STR) else {
                        return Err(XrnErrorKind::PathMissingTreePrefix
                            .build_message(format!("'{remain}' in {s}")));
                    };
                    let (remain, tree_part) = remain.split_at(id_pos);
                    let id_str = &tree_part[TREE_PREFIX_STR.len()..];
                    let id = id_str.parse::<u32>().map_err(
                        XrnErrorKind::PathTreeIdNotANumber.err_message_fn_map(|| {
                            format!("input '{id_str}' from part '{tree_part}' - {s}")
                        }),
                    )?;
                    Ok(Self(XrnMerge::Path(v), id, remain.to_string()))
                }
            },
        }
    }
}

impl Serialize for XrnAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

// impl Deserialize for XrnAddr {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let raw = String::deserialize(deserializer)?;
//
//     }
// }

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    strum::AsRefStr,
    //
)]
#[strum(serialize_all = "lowercase")]
pub enum XrnMerge {
    Space(SpaceXrnType),
    Path(PathXrnType),
}

impl XrnMerge {
    pub fn types_as_str(&self) -> (&str, &str) {
        match self {
            Self::Space(sub) => (self.as_ref(), sub.as_ref()),
            Self::Path(sub) => (self.as_ref(), sub.as_ref()),
        }
    }

    pub fn to_type(&self) -> XrnType {
        match self {
            Self::Space(_) => XrnType::Space,
            Self::Path(_) => XrnType::Path,
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    strum::AsRefStr,
    strum::EnumString,
    strum::VariantArray,
    strum::Display,
    //
)]
#[strum(serialize_all = "lowercase")]
pub enum XrnType {
    Space,
    Path,
}

impl XrnTypeImpl for XrnType {}

impl<'de> Deserialize<'de> for XrnAddr {
    fn deserialize<D>(deserializer: D) -> Result<XrnAddr, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <&str>::deserialize(deserializer)?;
        XrnAddr::from_str(raw).map_err(|e| D::Error::custom(format!("xrnaddr_serde {}", e)))
    }
}

#[cfg(test)]
mod test {
    use crate::defs::address::{XrnAddr, XrnAddrRef, XrnMerge, XrnType};
    use crate::defs::space_xrn::SpaceXrnType;
    use std::str::FromStr;
    use xana_commons_rs::PrettyUnwrap;

    #[test]
    fn enum_test() {
        assert_eq!(XrnType::Space.as_ref(), "space");

        let addr = XrnAddr::new(XrnMerge::Space(SpaceXrnType::Simple), 123, "proj");
        assert_eq!(addr.to_string(), "xrn:space:simple:123:proj")
    }

    #[test]
    fn parse_test() {
        let raw = "xrn:space:simple:123:proj";
        let addr = XrnAddr::from_str(raw).pretty_unwrap();
        assert_eq!(addr.merge(), XrnMerge::Space(SpaceXrnType::Simple));
        assert_eq!(addr.id(), 123);
        assert_eq!(addr.value(), "proj");
        assert_eq!(addr.to_string(), raw);
    }
}
