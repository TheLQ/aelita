use crate::StorDieselResult;
use crate::err::StorDieselErrorKind;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};
use xana_commons_rs::CrashErrKind;

pub fn convert_path_to_comps(path: &Path) -> StorDieselResult<Vec<&[u8]>> {
    let mut component_bytes = Vec::new();
    let mut components = path.components();
    let Some(root) = components.next() else {
        return Err(StorDieselErrorKind::EmptyPath.build_message(path.display()));
    };
    if root != Component::RootDir {
        return Err(StorDieselErrorKind::PathNotAbsolute.build_message(path.display()));
    }
    for component in components {
        let os_str = match component {
            Component::Normal(v) => v,
            _unknown => {
                return Err(StorDieselErrorKind::PathWeird.build_message(path.display()));
            }
        };
        component_bytes.push(os_str.as_bytes());
    }

    Ok(component_bytes)
}

pub fn convert_path_to_comps_owned(path: &Path) -> StorDieselResult<Vec<Vec<u8>>> {
    convert_path_to_comps(path).map(|v| v.into_iter().map(|v| v.to_vec()).collect())
}

#[cfg(test)]
mod test {
    use crate::convert_path_to_comps;
    use std::path::Path;

    #[test]
    fn is_component() {
        let path = Path::new("/foo/bar");
        let comp = convert_path_to_comps(path).unwrap();
        assert_eq!(comp, vec![b"foo", b"bar"]);

        let path = Path::new("/");
        let comp = convert_path_to_comps(path).unwrap();
        assert!(comp.is_empty());
    }
}
