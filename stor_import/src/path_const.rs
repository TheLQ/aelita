use std::ops::Deref;
use std::path::{Path, PathBuf};

/// Const [Path::new] is very unstable. `#![feature(const_convert)]` and `#![feature(const_trait_impl)]`
pub struct PathConst(pub &'static str);

impl Deref for PathConst {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        Path::new(self.0)
    }
}

impl AsRef<Path> for PathConst {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl Into<PathBuf> for PathConst {
    fn into(self) -> PathBuf {
        self.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use crate::path_const::PathConst;

    #[test]
    fn test() {
        const SUPER: PathConst = PathConst("super");
        let name = SUPER.to_path_buf().to_str().unwrap().to_string();
    }
}
