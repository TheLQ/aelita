use crate::defs::address::{XrnAddr, XrnMerge, XrnType};
use crate::defs::common::XrnTypeImpl;
use crate::err::{LibxrnError, XrnErrorKind};
use serde::{Serialize, Serializer};
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use xana_commons_rs::CrashErrKind;

pub(super) const TREE_PREFIX_STR: &str = "/__tree";

#[derive(Debug)]
pub struct PathXrn<'x>(&'x XrnAddr);

impl<'x> PathXrn<'x> {
    pub fn new(ptype: PathXrnType, path: PathBuf, tree_id: u32) -> XrnAddr {
        XrnAddr(
            XrnMerge::Path(ptype),
            tree_id,
            path.to_str().unwrap().to_string(),
        )
    }

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
        Path::new(self.0.value())
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

#[cfg(test)]
mod tests {
    use crate::defs::address::{XrnAddr, XrnMerge, XrnType};
    use crate::defs::path_xrn::{PathXrn, PathXrnType};
    use crate::err::test::assert_err_kind;
    use crate::err::{LibxrnResult, XrnErrorKind};
    use aelita_commons::log_init;
    use std::path::Path;
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
