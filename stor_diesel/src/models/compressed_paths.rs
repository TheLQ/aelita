use crate::StorDieselResult;
use crate::err::StorDieselErrorKind;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{info, trace, warn};
use xana_commons_rs::{CrashErrKind, LOCALE, ScanFileType, ScanFileTypeWithPath};

#[derive(Serialize)]
pub struct CompressedPathNested {
    parts: Vec<OsString>,
    nodes: CompNode,
}

impl CompressedPathNested {
    pub fn from_scan(scans: impl IntoIterator<Item = ScanFileTypeWithPath>) -> Self {
        let mut build = CompressedPathNestedBuilder::new();
        let mut cur_scan = 0;
        for scan in scans {
            if cur_scan % 100_000 == 0 {
                info!("compressing {}", cur_scan.to_formatted_string(&LOCALE))
            }
            cur_scan += 1;

            let (path, stype) = scan.into_parts();
            build.push_path(&path, stype)
        }
        Self::from_build(build)
    }

    fn from_build(
        CompressedPathNestedBuilder { parts, nodes }: CompressedPathNestedBuilder,
    ) -> Self {
        Self {
            parts: parts.into_iter().collect(),
            nodes: CompNode::from_builder(nodes, &mut PathBuf::from("/")),
        }
    }
}

struct CompressedPathNestedBuilder {
    parts: IndexSet<OsString>,
    nodes: CompNodeBuilder,
}

impl CompressedPathNestedBuilder {
    fn new() -> Self {
        Self {
            parts: IndexSet::new(),
            nodes: CompNodeBuilder {
                name: usize::MAX,
                node_type: None,
                children: Vec::new(),
            },
        }
    }

    fn push_path(&mut self, path: &Path, new_node_type: ScanFileType) {
        let mut comps = path.components();
        assert_eq!(comps.next(), Some(Component::RootDir));

        self.nodes
            .add_node_recursive(comps, new_node_type, &mut self.parts, path);
    }
}

struct CompNodeBuilder {
    name: usize,
    node_type: Option<ScanFileType>,
    children: Vec<CompNodeBuilder>,
}

impl CompNodeBuilder {
    pub fn add_node_recursive<'c>(
        &mut self,
        mut comps: impl Iterator<Item = Component<'c>>,
        new_node_type: ScanFileType,
        parts: &mut IndexSet<OsString>,
        debug_source: &Path,
    ) -> Option<ScanFileType> {
        let Some(next) = comps.next() else {
            return Some(new_node_type);
        };
        let Component::Normal(comp) = next else {
            panic!("Valid path {}", debug_source.display())
        };
        let comp_index = parts.get_index_of(comp).unwrap_or_else(|| {
            parts.insert(comp.to_os_string());
            parts.len() - 1
        });

        let next_level =
            if let Some(exist) = self.children.iter_mut().find(|v| v.name == comp_index) {
                exist
            } else {
                self.children.push(Self {
                    name: comp_index,
                    node_type: None,
                    children: Vec::new(),
                });
                self.children.last_mut().unwrap()
            };

        let is_last = next_level.add_node_recursive(comps, new_node_type, parts, debug_source);
        if let Some(new_node_type) = is_last {
            next_level.node_type = Some(new_node_type);
        }
        None
    }
}

#[derive(Serialize, Deserialize)]
struct CompNode {
    name: usize,
    node_type: ScanFileType,
    children: Vec<CompNode>,
}

impl CompNode {
    fn from_builder(
        CompNodeBuilder {
            name,
            node_type,
            children,
        }: CompNodeBuilder,
        debug_context: &mut PathBuf,
    ) -> Self {
        // debug_context.push(&name);
        let node_type = node_type.unwrap_or_else(|| {
            warn!("no node type for {} assume dir", debug_context.display());
            // if debug_context.iter().count() > 5 {
            //     panic!("uhh")
            // }
            ScanFileType::Dir
        });

        let children = children
            .into_iter()
            .map(|v| CompNode::from_builder(v, debug_context))
            .collect();
        // if debug_context != Path::new("/") {
        //     assert!(debug_context.pop(), "cur {}", debug_context.display());
        // }
        Self {
            name,
            node_type,
            children,
        }
    }
}

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

    fn push_path(&mut self, path: &Path) -> StorDieselResult<()> {
        let mut indexed_path = Vec::new();

        let mut components = path.components();

        match components.next() {
            Some(Component::RootDir) => {
                // expected
            }
            Some(bad) => {
                return Err(StorDieselErrorKind::CompressedPathNotRoot
                    .build_message(format!("unknown root {bad:?}")));
            }
            None => return Err(StorDieselErrorKind::CompressedPathEmpty.build()),
        }

        for (i, next) in components.enumerate() {
            let part = match next {
                Component::Normal(part) => part.to_str().unwrap(),
                bad => {
                    return Err(StorDieselErrorKind::CompressedUnknownComponent
                        .build_message(format!("unknown {bad:?} at {i} in {}", path.display())));
                }
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
    indexed_paths: Vec<(Vec<u32>)>,
}

impl CompressedPaths {
    pub fn from_paths(
        paths: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> StorDieselResult<CompressedPaths> {
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
