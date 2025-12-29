use crate::StorDieselResult;
use crate::err::StorDieselErrorKind;
use indexmap::IndexSet;
use rayon::prelude::ParallelSliceMut;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};
use xana_commons_rs::tracing_re::{info, warn};
use xana_commons_rs::{CrashErrKind, ProgressWidget, ScanFileType, ScanFileTypeWithPath};

#[derive(Serialize)]
pub struct CompressedPathNested {
    parts: Vec<OsString>,
    nodes: Vec<CompNode>,
}

impl CompressedPathNested {
    pub fn from_scan(mut scans: Vec<ScanFileTypeWithPath>) -> Self {
        info!("starting sort");
        scans.par_sort();

        let mut build = CompressedPathNestedBuilder::new();
        let total_scans = scans.len();
        let mut progress = ProgressWidget::new(4096);
        for (i, scan) in scans.into_iter().enumerate() {
            progress.log(i, total_scans, |msg| info!("scan import {msg}"));

            let (path, stype) = scan.into_parts();
            build.push_path(&path, stype)
        }
        Self::from_build(build)
    }

    fn from_build(
        CompressedPathNestedBuilder {
            parts,
            nodes,
            cache: _,
        }: CompressedPathNestedBuilder,
    ) -> Self {
        Self {
            parts: parts.into_iter().collect(),
            nodes: nodes
                .into_iter()
                .map(|v| CompNode::from_builder(v, &mut PathBuf::from("/")))
                .collect(),
        }
    }
}
struct CompressedPathNestedBuilder {
    parts: IndexSet<OsString>,
    nodes: Vec<CompNodeBuilder>,
    /// By pre-sorting the input paths we can cache eg 9/10 components
    /// Vastly improving performance with 30 million paths
    /// 10k/sec to 250k/sec
    cache: Vec<CachedLookup>,
}

impl CompressedPathNestedBuilder {
    fn new() -> Self {
        Self {
            parts: IndexSet::new(),
            nodes: vec![CompNodeBuilder {
                node_type: None,
                children_comp_ids: Vec::new(),
                children_indexes: Vec::new(),
            }],
            cache: Vec::new(),
        }
    }

    fn push_path(&mut self, path: &Path, new_node_type: ScanFileType) {
        let mut comps = path.components();
        assert_eq!(comps.next(), Some(Component::RootDir));
        self.add_node_linear(0, comps, new_node_type, path)
    }

    fn add_node_linear<'c>(
        &mut self,
        start_index: usize,
        comps: impl Iterator<Item = Component<'c>>,
        new_node_type: ScanFileType,
        debug_source: &Path,
    ) {
        let mut last_index = start_index;
        for (i, comp) in comps.enumerate() {
            let Component::Normal(comp) = comp else {
                panic!("Valid path {}", debug_source.display())
            };

            let mut is_clear_cache = false;
            if let Some(cache) = self.cache.get(i) {
                if cache.component == comp {
                    // yay
                    last_index = cache.child_id;
                    continue;
                } else {
                    is_clear_cache = true;
                }
            }
            if is_clear_cache {
                self.cache.truncate(i);
            }

            let comp_index = self.parts.get_index_of(comp).unwrap_or_else(|| {
                self.parts.insert(comp.to_os_string());
                self.parts.len() - 1
            });

            if let Some(pos) = self.nodes[last_index]
                .children_comp_ids
                .iter()
                .rposition(|v| *v == comp_index)
            {
                last_index = self.nodes[last_index].children_indexes[pos];
            } else {
                let next_index = self.nodes.len();
                self.nodes[last_index].children_comp_ids.push(comp_index);
                self.nodes[last_index].children_indexes.push(next_index);
                self.nodes.push(CompNodeBuilder {
                    node_type: None,
                    children_comp_ids: Vec::new(),
                    children_indexes: Vec::new(),
                });
                last_index = next_index;
            };

            self.cache.push(CachedLookup {
                child_id: last_index,
                component: comp.to_os_string(),
            })
        }
        let last = &mut self.nodes[last_index];
        last.node_type = Some(new_node_type);
    }
}

struct CompNodeBuilder {
    node_type: Option<ScanFileType>,
    children_indexes: Vec<usize>,
    children_comp_ids: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
struct CompNode {
    node_type: ScanFileType,
    children_indexes: Vec<usize>,
}

impl CompNode {
    fn from_builder(
        CompNodeBuilder {
            node_type,
            children_indexes,
            children_comp_ids,
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
        // if debug_context != Path::new("/") {
        //     assert!(debug_context.pop(), "cur {}", debug_context.display());
        // }
        Self {
            node_type,
            children_indexes,
        }
    }
}

struct CachedLookup {
    component: OsString,
    child_id: usize,
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
