use crate::defs::address::{XrnAddr, XrnType};
use crate::err::LibxrnError;
use std::backtrace::Backtrace;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathXrn {
    path: PathBuf,
    tree_id: Option<u32>,
}

impl PathXrn {
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn tree_id(&self) -> &Option<u32> {
        &self.tree_id
    }
}

impl TryFrom<XrnAddr> for PathXrn {
    type Error = LibxrnError;

    fn try_from(addr: XrnAddr) -> Result<Self, Self::Error> {
        if addr.atype() != &XrnType::Path {
            return Err(LibxrnError::AddrInvalidType(addr, Backtrace::capture()));
        }

        let value = addr.value();
        let mut path = PathBuf::from(value);

        const TREE_PREFIX_BYTES: &[u8] = b"__tree";

        let tree_id;
        let last_component = path.iter().last().unwrap();
        if last_component.len() <= TREE_PREFIX_BYTES.len() {
            tree_id = None;
        } else if &last_component.as_bytes()[0..TREE_PREFIX_BYTES.len()] == TREE_PREFIX_BYTES {
            let id_raw = &last_component.as_bytes()[TREE_PREFIX_BYTES.len()..];
            let id_str = str::from_utf8(id_raw).unwrap();
            match id_str.parse::<u32>() {
                Ok(v) => {
                    tree_id = Some(v);
                    path.pop();
                }
                Err(e) => {
                    return Err(LibxrnError::InvalidType(
                        format!("not a number {e}"),
                        Backtrace::capture(),
                    ));
                }
            };
        } else {
            tree_id = None;
        }

        Ok(Self { path, tree_id })
    }
}
