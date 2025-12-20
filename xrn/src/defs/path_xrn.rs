use crate::defs::address::{XrnAddr, XrnType};
use crate::defs::common::{SubXrn, XrnTypeImpl};
use crate::err::{LibxrnError, XrnErrorKind};
use serde::{Serialize, Serializer};
use std::fmt::Formatter;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathXrn {
    ptype: PathXrnType,
    path: PathBuf,
    tree_id: Option<u32>,
}

impl PathXrn {
    pub fn from_components(comp: &[String]) -> Self {
        let mut path = PathBuf::from("/");
        path.extend(comp);
        Self::from_path(path)
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        assert!(path.is_absolute());
        Self {
            path,
            ptype: PathXrnType::Fs,
            tree_id: None,
        }
    }

    pub fn ptype(&self) -> PathXrnType {
        self.ptype
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
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
    fn from(value: &PathXrn) -> Self {
        XrnAddr::new(XrnType::Path, value.path().to_str().unwrap())
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

        const TREE_PREFIX_BYTES: &[u8] = b"__tree";
        let tree_id;
        let mut path = PathBuf::from(remain);
        if let Some((prefix, remain)) = path
            .iter()
            .last()
            // safety: the path isn't empty so must have something
            .unwrap()
            .as_bytes()
            .split_at_checked(TREE_PREFIX_BYTES.len())
            && prefix == TREE_PREFIX_BYTES
        {
            let Ok(remain) = str::from_utf8(remain) else {
                return Err(XrnErrorKind::InvalidTreeId.err_addr(addr));
            };
            let Ok(id) = remain.parse::<u32>() else {
                return Err(XrnErrorKind::InvalidTreeId.err_addr(addr));
            };
            path.pop();
            tree_id = Some(id);
        } else {
            tree_id = None;
        }

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
        assert_eq!(path.path(), Path::new("/test/hello"));
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
        assert_eq!(path.path(), Path::new("/test/hello"));
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
        assert_eq!(path.ptype(), PathXrnType::Fs);
        assert_eq!(path.path(), Path::new("/test/hello"));
        assert_eq!(path.tree_id(), Some(498));
        Ok(path)
    }
}
