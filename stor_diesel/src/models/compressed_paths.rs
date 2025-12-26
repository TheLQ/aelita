use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
struct CompressedPathsBuilder {
    parts: IndexSet<String>,
    indexed_paths: Vec<Vec<u32>>,
}
impl CompressedPathsBuilder {
    fn new() -> Self {
        Self {
            parts: IndexSet::new(),
            indexed_paths: Vec::new(),
        }
    }

    fn push_path(&mut self, path: &Path) -> Result<(), String> {
        let mut indexed_path = Vec::new();

        let mut components = path.components();

        match components.next() {
            Some(Component::RootDir) => {
                // expected
            }
            Some(bad) => return Err(format!("unknown root {bad:?}")),
            None => return Err("path empty?".into()),
        }

        for (i, next) in components.enumerate() {
            let part = match next {
                Component::Normal(part) => part.to_str().unwrap(),
                bad => return Err(format!("unknown {bad:?} at {i}")),
            };

            match self.parts.get_index_of(part) {
                Some(index) => indexed_path.push(u32::try_from(index).unwrap()),
                None => {
                    let index = self.parts.len();
                    self.parts.insert(part.to_string());
                    indexed_path.push(u32::try_from(index).unwrap());
                }
            }
        }

        self.indexed_paths.push(indexed_path);
        Ok(())
    }

    fn build(self) -> CompressedPaths {
        let Self {
            parts,
            indexed_paths,
        } = self;
        CompressedPaths {
            parts: parts.into_iter().collect(),
            indexed_paths,
        }
    }
}

/// 30 million files saved
#[derive(Serialize, Deserialize)]
pub struct CompressedPaths {
    parts: Vec<String>,
    indexed_paths: Vec<Vec<u32>>,
}

impl CompressedPaths {
    pub fn from_paths(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<Self, String> {
        let mut builder = CompressedPathsBuilder::new();
        for path in paths {
            let path = path.as_ref();
            builder.push_path(path)?;
        }
        Ok(builder.build())
    }

    pub fn iter_paths(&self) -> impl Iterator<Item = PathBuf> {
        self.indexed_paths.iter().map(|indexes| {
            // make default relative an absolute path
            let mut root = PathBuf::from("/");
            root.extend(indexes.iter().map(|i| &self.parts[*i as usize]));
            root
        })
    }

    pub fn parts(&self) -> &[String] {
        self.parts.as_slice()
    }

    pub fn inner(&self) -> (&[String], &[Vec<u32>]) {
        (&self.parts, &self.indexed_paths)
    }
}

#[cfg(test)]
mod test {
    use super::CompressedPathsBuilder;
    use std::path::Path;

    #[test]
    fn basic() {
        let mut builder = CompressedPathsBuilder::new();
        builder.push_path(Path::new("/head/content")).unwrap();
        builder.push_path(Path::new("/nope/test")).unwrap();
        builder.push_path(Path::new("/head/other")).unwrap();

        let compressed = builder.build();
        let mut iter = compressed.iter_paths();
        assert_eq!(iter.next(), Some(Path::new("/head/content").to_path_buf()));
        assert_eq!(iter.next(), Some(Path::new("/nope/test").to_path_buf()));
        assert_eq!(iter.next(), Some(Path::new("/head/other").to_path_buf()));
        assert_eq!(iter.next(), None);
    }
}
