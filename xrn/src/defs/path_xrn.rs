use crate::defs::address::{XrnAddr, XrnType};
use crate::defs::common::{SubXrn, XrnTypeImpl};
use crate::err::{LibxrnError, XrnErrorKind};
use serde::{Serialize, Serializer};
use std::fmt::Formatter;
use std::fs::rename;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};

const TREE_PREFIX_STR: &str = "/__tree";

#[derive(Debug, Clone)]
pub struct PathXrn {
    ptype: PathXrnType,
    path: Option<PathBuf>,
    tree_id: Option<u32>,
}

impl PathXrn {
    pub fn new_path(ptype: PathXrnType, path: impl Into<PathBuf>) -> Self {
        Self {
            ptype,
            path: Some(path.into()),
            tree_id: None,
        }
    }

    pub fn new_path_id(ptype: PathXrnType, path: PathBuf, tree_id: u32) -> Self {
        Self {
            ptype,
            path: Some(path),
            tree_id: Some(tree_id),
        }
    }

    pub fn new_id(ptype: PathXrnType, tree_id: u32) -> Self {
        Self {
            ptype,
            path: None,
            tree_id: Some(tree_id),
        }
    }

    pub fn from_components(comp: &[String]) -> Self {
        let mut path = PathBuf::from("/");
        path.extend(comp);
        Self::from_path(path)
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        assert!(path.is_absolute());
        Self {
            path: Some(path.into()),
            ptype: PathXrnType::Fs,
            tree_id: None,
        }
    }

    pub fn ptype(&self) -> PathXrnType {
        self.ptype
    }

    // pub fn path_unwrap(&self) -> &Path {
    //     self.path.as_ref().unwrap()
    // }
    pub fn path(&self) -> Option<&Path> {
        self.path.as_ref().map(|v| v.as_path())
    }

    pub fn tree_id(&self) -> Option<u32> {
        self.tree_id
    }
}

impl SubXrn for PathXrn {
    fn atype() -> XrnType {
        XrnType::Path
    }
}

impl From<PathXrn> for XrnAddr {
    fn from(value: PathXrn) -> Self {
        XrnAddr::from(&value)
    }
}

impl From<&PathXrn> for XrnAddr {
    fn from(input: &PathXrn) -> Self {
        let mut value = String::new();
        if let Some(path) = &input.path {
            value.push_str(path.to_str().unwrap())
        }
        if let Some(tree_id) = &input.tree_id {
            value.push_str(TREE_PREFIX_STR);
            value.push_str(&tree_id.to_string());
        }
        XrnAddr::new(XrnType::Path, value)
    }
}

impl TryFrom<XrnAddr> for PathXrn {
    type Error = LibxrnError;

    fn try_from(addr: XrnAddr) -> Result<Self, Self::Error> {
        if addr.atype() != XrnType::Path {
            return Err(XrnErrorKind::PathInvalidInputType.err_addr(addr));
        }
        let value = addr.value();

        let (ptype, remain) = match PathXrnType::split_type(value) {
            None => return Err(XrnErrorKind::PathInvalidType.err_addr(addr)),
            Some((_, "")) => {
                return Err(XrnErrorKind::PathEmptyValue.err_addr(addr));
            }
            Some(v) => v,
        };

        let tree_id;
        let remain = if let Some(tree_start) = remain.find(TREE_PREFIX_STR) {
            let tree_len = TREE_PREFIX_STR.len();
            let (value_remain, cur_remain) = remain.split_at_checked(tree_start).unwrap();

            let (_, cur_remain) = cur_remain.split_at_checked(tree_len).unwrap();
            let Ok(tree_id_raw) = cur_remain.parse::<u32>() else {
                return Err(XrnErrorKind::InvalidTreeId.err_addr(addr));
            };
            tree_id = Some(tree_id_raw);
            value_remain
        } else {
            tree_id = None;
            remain
        };

        let path = if remain.is_empty() {
            None
        } else {
            Some(PathBuf::from(remain))
        };

        Ok(Self {
            path,
            ptype,
            tree_id,
        })
    }
}

impl Serialize for PathXrn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(XrnAddr::from(self).to_string().as_str())
    }
}

impl std::fmt::Display for PathXrn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let addr: XrnAddr = self.into();
        addr.fmt(f)
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

impl XrnTypeImpl<'_> for PathXrnType {}

#[cfg(test)]
mod tests {
    use crate::defs::address::{XrnAddr, XrnType};
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

    fn _good_path() -> LibxrnResult<PathXrn> {
        let addr = XrnAddr::from_str("xrn:path:fs/test/hello")?;
        assert_eq!(addr.atype(), XrnType::Path);
        assert_eq!(addr.value(), ":fs/test/hello");

        let path: PathXrn = addr.try_into()?;
        assert_eq!(path.ptype(), PathXrnType::Fs);
        assert_eq!(path.path(), Some(Path::new("/test/hello")));
        assert_eq!(path.tree_id(), None);
        Ok(path)
    }

    #[test]
    fn empty_path() {
        log_init();
        assert_err_kind(_empty_path(), XrnErrorKind::PathEmptyValue)
    }

    fn _empty_path() -> LibxrnResult<PathXrn> {
        let addr = XrnAddr::from_str("xrn:path:fs")?;
        assert_eq!(addr.atype(), XrnType::Path);
        assert_eq!(addr.value(), ":fs");

        addr.try_into()
    }

    //

    #[test]
    fn good_path_tree_id() {
        log_init();
        _good_path_tree_id().pretty_unwrap();
    }

    fn _good_path_tree_id() -> LibxrnResult<PathXrn> {
        let path: PathXrn = XrnAddr::from_str("xrn:path:fs/test/hello/__tree498")?.try_into()?;
        assert_eq!(path.ptype(), PathXrnType::Fs);
        assert_eq!(path.path(), Some(Path::new("/test/hello")));
        assert_eq!(path.tree_id(), Some(498));
        Ok(path)
    }

    #[test]
    fn empty_tree_id() {
        log_init();
        assert_err_kind(_empty_tree_id(), XrnErrorKind::InvalidTreeId)
    }

    fn _empty_tree_id() -> LibxrnResult<PathXrn> {
        let path: PathXrn = XrnAddr::from_str("xrn:path:fs/test/hello/__tree")?.try_into()?;
        Ok(path)
    }

    #[test]
    fn only_tree_id() {
        log_init();
        assert_err_kind(_empty_tree_id(), XrnErrorKind::InvalidTreeId)
    }

    fn _only_tree_id() -> LibxrnResult<PathXrn> {
        let path: PathXrn = XrnAddr::from_str("xrn:path:fs/__tree6599")?.try_into()?;
        assert_eq!(path.ptype(), PathXrnType::Fs);
        assert_eq!(path.path(), None);
        assert_eq!(path.tree_id(), Some(6599));
        Ok(path)
    }
}
