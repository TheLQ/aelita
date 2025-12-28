use crate::defs::address::{XrnAddr, XrnAddrRef, XrnMerge, XrnType};
use crate::defs::common::{SubXrnImpl, XrnSubTypeImpl, XrnTypeImpl, check_expected_type};
use crate::err::LibxrnError;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub(super) const TREE_PREFIX_STR: &str = "/__tree";
pub const XRN_PATH_ROOT_ID: u32 = u32::MAX - 166;

#[derive(Debug, Clone)]
pub struct PathXrn(XrnAddr);

impl PathXrn {
    pub fn new(ptype: PathXrnType, path: PathBuf, tree_id: u32) -> XrnAddr {
        XrnAddr(
            XrnMerge::Path(ptype),
            tree_id,
            path.to_str().unwrap().to_string(),
        )
    }

    // todo how often is this?
    // pub fn new_no_path(ptype: PathXrnType, tree_id: u32) -> XrnAddr {
    //     XrnAddr(XrnMerge::Path(ptype), tree_id, String::new())
    // }

    // pub fn from_components(comp: &[String]) -> Self {
    //     let mut path = PathBuf::from("/");
    //     path.extend(comp);
    //     Self::from_path(path)
    // }

    // pub fn from_path(path: impl Into<PathBuf>) -> Self {
    //     let path = path.into();
    //     assert!(path.is_absolute());
    //     Self {
    //         path: Some(path.into()),
    //         ptype: PathXrnType::Fs,
    //         tree_id: None,
    //     }
    // }

    pub fn path(&self) -> &Path {
        Path::new(self.value())
    }
}

impl XrnAddrRef for PathXrn {
    fn addr_ref(&self) -> &XrnAddr {
        &self.0
    }
}

impl SubXrnImpl for PathXrn {
    const UPPER: XrnType = XrnType::Path;
    type SubXrnType = PathXrnType;

    fn sub_type(&self) -> Self::SubXrnType {
        let XrnMerge::Path(kind) = self.addr_ref().merge() else {
            panic!("wut")
        };
        kind
    }
}

impl TryFrom<XrnAddr> for PathXrn {
    type Error = Box<LibxrnError>;
    fn try_from(addr: XrnAddr) -> Result<Self, Self::Error> {
        check_expected_type(Self::UPPER, &addr)?;
        Ok(Self(addr))
    }
}

impl FromStr for PathXrn {
    type Err = Box<LibxrnError>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr = XrnAddr::from_str(s)?;
        addr.try_into()
    }
}

impl From<PathXrn> for XrnAddr {
    fn from(value: PathXrn) -> Self {
        value.0
    }
}

impl Display for PathXrn {
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
    //
)]
#[strum(serialize_all = "lowercase")]
pub enum PathXrnType {
    Fs,
    Volume,
    Mount,
}

impl XrnTypeImpl for PathXrnType {}

impl XrnSubTypeImpl for PathXrnType {}

#[cfg(test)]
mod tests {
    use crate::defs::address::{XrnAddr, XrnAddrRef, XrnMerge};
    use crate::defs::path_xrn::PathXrnType;
    use crate::err::test::assert_err_kind;
    use crate::err::{LibxrnResult, XrnErrorKind};
    use aelita_commons::log_init;
    use std::str::FromStr;
    use xana_commons_rs::PrettyUnwrap;

    #[test]
    fn good_path() {
        log_init();
        _good_path().pretty_unwrap();
    }

    fn _good_path() -> LibxrnResult<XrnAddr> {
        let input = "xrn:path:fs/test/hello/__tree999";
        let addr = XrnAddr::from_str(input)?;
        assert_eq!(addr.merge(), XrnMerge::Path(PathXrnType::Fs));
        assert_eq!(addr.value(), "/test/hello");
        assert_eq!(addr.id(), 999);
        assert_eq!(&addr.to_string(), input);
        Ok(addr)
    }

    #[test]
    fn empty_path() {
        log_init();
        assert_err_kind(_empty_path(), XrnErrorKind::PathMissingTreePrefix)
    }

    fn _empty_path() -> LibxrnResult<XrnAddr> {
        let addr = XrnAddr::from_str("xrn:path:fs")?;
        assert_eq!(addr.merge(), XrnMerge::Path(PathXrnType::Fs));
        assert_eq!(addr.value(), "");
        Ok(addr)
    }

    //

    #[test]
    fn good_path_tree_id() {
        log_init();
        let addr = XrnAddr::from_str("xrn:path:fs/test/hello/__tree498").pretty_unwrap();
        assert_eq!(addr.merge(), XrnMerge::Path(PathXrnType::Fs));
        assert_eq!(addr.value(), "/test/hello");
        assert_eq!(addr.id(), 498);
    }

    #[test]
    fn empty_tree_id() {
        log_init();
        assert_err_kind(
            XrnAddr::from_str("xrn:path:fs/test/hello/__tree"),
            XrnErrorKind::PathTreeIdNotANumber,
        );
    }

    #[test]
    fn only_tree_id() {
        log_init();
        let addr = XrnAddr::from_str("xrn:path:fs/__tree6599").pretty_unwrap();
        assert_eq!(addr.merge(), XrnMerge::Path(PathXrnType::Fs));
        assert_eq!(addr.value(), "");
        assert_eq!(addr.id(), 6599);
    }
}
