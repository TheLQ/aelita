use crate::StorDieselResult;
use crate::err::StorDieselErrorKind;
use std::ffi::OsStr;
use std::fmt::Formatter;
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

pub fn convert_comps_to_path(comps: &[impl AsRef<[u8]>]) -> PathBuf {
    let mut res = PathBuf::from("/");
    res.extend(comps.iter().map(|v| OsStr::from_bytes(v.as_ref())));
    res
}

pub fn convert_comps_to_list(comps: &[impl AsRef<[u8]>]) -> String {
    // we could use opt.unwrap_or_else("NOT_UTF8") but lets see what this does
    struct Wrapper<'v, I: AsRef<[u8]>> {
        inner: &'v [I],
    }
    impl<'v, I: AsRef<[u8]>> std::fmt::Display for Wrapper<'v, I> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            for comp in self.inner {
                let comp = comp.as_ref();
                write!(f, "{}", OsStr::from_bytes(comp).display())?;
            }
            Ok(())
        }
    }

    Wrapper { inner: comps }.to_string()
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
