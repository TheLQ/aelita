use crate::StorDieselResult;
use crate::err::StorDieselErrorKind;
use indexmap::IndexSet;
use rayon::prelude::ParallelSliceMut;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::ops::ControlFlow;
use std::path::{Component, Path, PathBuf};
use xana_commons_rs::tracing_re::{info, trace, warn};
use xana_commons_rs::{CrashErrKind, ProgressWidget, ScanFileType, ScanFileTypeWithPath};

#[derive(Debug, Serialize)]
pub struct CompressedPathNested {
    parts: Vec<OsString>,
    nodes: Vec<CompNode>,
}

impl CompressedPathNested {
    pub fn from_scan(mut scans: Vec<ScanFileTypeWithPath>) -> StorDieselResult<Self> {
        info!("starting sort");
        scans.par_sort();

        let mut build = CompressedPathNestedBuilder::new();
        let total_scans = scans.len();
        let mut progress = ProgressWidget::new(4096);
        for (i, scan) in scans.into_iter().enumerate() {
            progress.log(i, total_scans, |msg| info!("scan import {msg}"));

            let (path, stype) = scan.into_parts();
            info!("scanning {}", path.display());
            build.push_path(&path, stype)
        }
        Self::from_build(build)
    }

    fn from_build(builder: CompressedPathNestedBuilder) -> StorDieselResult<Self> {
        let nodes = (0..builder.nodes.len())
            .into_iter()
            .map(|i| CompNode::from_builder(&builder, i, &mut PathBuf::from("/")))
            .try_collect()?;
        Ok(Self {
            parts: builder.parts.into_iter().collect(),
            nodes,
        })
    }

    pub fn iter_path_vecs<'i>(&'i self) -> CompressedIterFiles<'i> {
        CompressedIterFiles::new(self)
    }

    // todo generic between builder and main
    fn find_node_parent(&self, node_id: usize) -> usize {
        self.nodes
            .par_iter()
            .position_any(|v| {
                if let CompNodeType::Dir { children_node_ids } = &v.node_type {
                    children_node_ids.contains(&node_id)
                } else {
                    false
                }
            })
            .unwrap()
    }

    // todo generic between builder and main
    fn path_vec_from_node_id(&self, node_id: usize) -> Vec<&OsStr> {
        let mut path_rev = Vec::new();
        let mut next_id = node_id;
        loop {
            let cur_node = &self.nodes[next_id];
            path_rev.push(self.parts[cur_node.name_comp_id].as_os_str());
            next_id = self.find_node_parent(next_id);
            if next_id == 0 {
                break;
            }
        }
        path_rev.reverse();
        path_rev
    }
}
struct CompressedPathNestedBuilder {
    parts: IndexSet<OsString>,
    nodes: Vec<CompNodeBuilder>,
    /// By pre-sorting the input paths we can cache eg 9/10 components
    /// Vastly improving performance with 30 million paths up to 9 levels deep
    /// 10k/sec to 250k/sec
    cache: Vec<CachedLookup>,
}

impl CompressedPathNestedBuilder {
    fn new() -> Self {
        Self {
            parts: IndexSet::new(),
            nodes: vec![CompNodeBuilder {
                name_comp_id: usize::MAX - 100,
                node_type: None,
                children_indexes: Vec::new(),
                delayed_symlink: None,
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

            if let Some(node_id) = self.find_node_children(last_index, comp_index) {
                last_index = node_id;
            } else {
                let next_index = self.nodes.len();
                self.nodes[last_index].children_indexes.push(next_index);
                self.nodes.push(CompNodeBuilder {
                    name_comp_id: comp_index,
                    node_type: None,
                    children_indexes: Vec::new(),
                    delayed_symlink: None,
                });
                last_index = next_index;
            };

            self.cache.push(CachedLookup {
                child_id: last_index,
                component: comp.to_os_string(),
            })
        }
        let last = &mut self.nodes[last_index];

        match new_node_type {
            ScanFileType::Dir => {
                last.node_type = Some(CompNodeType::Dir {
                    children_node_ids: Vec::new(),
                })
            }
            ScanFileType::File => last.node_type = Some(CompNodeType::File),
            ScanFileType::Symlink { target } => last.delayed_symlink = Some(target),
        };
    }

    fn find_node_children(&self, node_id: usize, needle_comp_id: usize) -> Option<usize> {
        self.nodes[node_id]
            .children_indexes
            .iter()
            .find(|v| self.nodes[**v].name_comp_id == needle_comp_id)
            .map(|v| *v)
    }

    // todo generic between builder and main
    fn find_node_parent(&self, node_id: usize) -> usize {
        self.nodes
            .par_iter()
            .position_any(|v| v.children_indexes.contains(&node_id))
            .unwrap()
    }

    // todo generic between builder and main
    fn path_vec_from_node_id(&self, node_id: usize) -> Vec<&OsString> {
        let mut path_rev = Vec::new();
        let mut next_id = node_id;
        loop {
            let cur_node = &self.nodes[next_id];
            path_rev.push(&self.parts[cur_node.name_comp_id]);
            next_id = self.find_node_parent(next_id);
            if next_id == 0 {
                break;
            }
        }
        path_rev.reverse();
        path_rev
    }

    fn pathbuf_from_node_id(&self, node_id: usize) -> PathBuf {
        self.path_vec_from_node_id(node_id).into_iter().collect()
    }

    fn path_to_node_id(&self, path: &Path) -> StorDieselResult<usize> {
        let mut comps = path.components();
        assert_eq!(comps.next(), Some(Component::RootDir));
        let mut next_id = 0;
        for comp in comps {
            let Component::Normal(comp) = comp else {
                return Err(StorDieselErrorKind::UnknownComponent.build_message(path.display()));
            };
            let Some(comp_id) = self.parts.get_index_of(comp) else {
                return Err(StorDieselErrorKind::UnknownComponent.build_message(path.display()));
            };
            let Some(child_id) = self.find_node_children(next_id, comp_id) else {
                return Err(StorDieselErrorKind::UnknownComponent.build_message(path.display()));
            };
            next_id = child_id;
        }
        Ok(next_id)
    }
}

struct CompNodeBuilder {
    name_comp_id: usize,
    node_type: Option<CompNodeType>,
    children_indexes: Vec<usize>,
    delayed_symlink: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompNode {
    name_comp_id: usize,
    node_type: CompNodeType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum CompNodeType {
    Dir { children_node_ids: Vec<usize> },
    File,
    Symlink { target_node_id: usize },
    BrokenSymlink { raw: String },
}

impl CompNode {
    fn from_builder(
        compressed_builder: &CompressedPathNestedBuilder,
        node_id: usize,
        debug_context: &mut PathBuf,
    ) -> StorDieselResult<Self> {
        let CompNodeBuilder {
            name_comp_id,
            node_type,
            children_indexes,
            delayed_symlink,
        } = &compressed_builder.nodes[node_id];
        let mut new_node_type = None;
        // debug_context.push(&name);
        if let Some(delayed_symlink) = delayed_symlink {
            assert!(node_type.is_none(), "{node_type:?}");
            if delayed_symlink.is_absolute() {
                let mut comps = delayed_symlink.components();
                assert_eq!(comps.next(), Some(Component::RootDir));

                let mut last_index = Some(0);
                while let Some(cur_last_index) = last_index {
                    let Some(comp) = comps.next() else {
                        break;
                    };
                    let Component::Normal(comp) = comp else {
                        // todo unlike input paths, symlinks can validly do weird things
                        last_index = None;
                        break;
                    };
                    let Some(comp_id) = compressed_builder.parts.get_index_of(comp) else {
                        // don't even have a name for it. bad
                        last_index = None;
                        break;
                    };
                    if let Some(node_id) =
                        compressed_builder.find_node_children(cur_last_index, comp_id)
                    {
                        last_index = Some(node_id)
                    } else {
                        last_index = None;
                        break;
                    };
                }
                if let Some(target_node_id) = last_index {
                    new_node_type = Some(CompNodeType::Symlink { target_node_id })
                }
            } else {
                let cur_path: PathBuf = compressed_builder
                    .path_vec_from_node_id(node_id)
                    .into_iter()
                    .collect();
                let new_path_raw = cur_path.join(delayed_symlink);
                let new_path = new_path_raw.normalize_lexically().map_err(|_marker_e| {
                    StorDieselErrorKind::SymlinkResolveFailed.build_message(new_path_raw.display())
                })?;
                let ref_id = compressed_builder.path_to_node_id(&new_path)?;
                new_node_type = Some(CompNodeType::Symlink {
                    target_node_id: ref_id,
                })
                // this path should now resolve
            }
        }
        // if debug_context != Path::new("/") {
        //     assert!(debug_context.pop(), "cur {}", debug_context.display());
        // }
        let res_node_type = if let Some(new_node_type) = new_node_type {
            new_node_type
        } else if let Some(node_type) = node_type.clone() {
            node_type
        } else {
            if children_indexes.is_empty() {
                trace!(
                    "assuming empty(!!!) dir for id {node_id} path {}",
                    compressed_builder.pathbuf_from_node_id(node_id).display()
                );
                CompNodeType::Dir {
                    children_node_ids: Vec::new(),
                }
            } else {
                trace!(
                    "assuming dir with {} children for id {node_id} path {}",
                    children_indexes.len(),
                    compressed_builder.pathbuf_from_node_id(node_id).display()
                );
                CompNodeType::Dir {
                    children_node_ids: children_indexes.clone(),
                }
            }
        };
        Ok(Self {
            name_comp_id: *name_comp_id,
            node_type: res_node_type,
        })
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
    indexed_paths: Vec<Vec<u32>>,
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

pub struct CompressedIterFiles<'c> {
    backend: &'c CompressedPathNested,
    cursor_stack: Vec<IterStack>,
}

impl<'c> CompressedIterFiles<'c> {
    fn new(backend: &'c CompressedPathNested) -> Self {
        Self {
            backend,
            cursor_stack: vec![IterStack {
                node_id: 0,
                next_child: 0,
            }],
        }
    }

    fn pre_advance(&mut self) -> ControlFlow<()> {
        let Some(IterStack {
            node_id,
            next_child,
        }) = self.cursor_stack.last().cloned()
        else {
            return ControlFlow::Break(());
        };

        let CompNodeType::Dir { children_node_ids } = &self.backend.nodes[node_id].node_type else {
            unreachable!("why am I not in a dir?")
        };
        if next_child >= children_node_ids.len() {
            self.cursor_stack.pop();
            return self.pre_advance();
        } else {
            let last = self.cursor_stack.last_mut().unwrap();
            last.next_child += 1;
        }

        let child_id = children_node_ids[next_child];
        match &self.backend.nodes[child_id].node_type {
            CompNodeType::Dir { .. } => {
                //
                self.cursor_stack.push(IterStack {
                    node_id: child_id,
                    next_child: 0,
                });
                self.pre_advance()
            }
            CompNodeType::File => {
                // found
                ControlFlow::Continue(())
            }
            CompNodeType::Symlink { target_node_id } => {
                trace!("ignore unsupported symlink {target_node_id}");
                self.pre_advance()
            }
            CompNodeType::BrokenSymlink { raw } => {
                trace!("ignore unsupported broken symlink {raw}");
                self.pre_advance()
            }
        }
    }
}

impl<'c> Iterator for CompressedIterFiles<'c> {
    type Item = Vec<&'c OsStr>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pre_advance().is_break() {
            return None;
        }
        let IterStack {
            node_id,
            next_child,
        } = self.cursor_stack.last().unwrap();
        let CompNodeType::Dir { children_node_ids } = &self.backend.nodes[*node_id].node_type
        else {
            unreachable!("how??")
        };

        let next_node = children_node_ids[*next_child];
        let path = self.backend.path_vec_from_node_id(next_node);
        Some(path)
    }
}

#[derive(Clone)]
struct IterStack {
    node_id: usize,
    next_child: usize,
}

#[cfg(test)]
mod test {
    use super::CompressedPathNested;
    use std::ffi::OsStr;
    use std::path::PathBuf;
    use xana_commons_rs::{PrettyUnwrap, ScanFileTypeWithPath};

    #[test]
    fn basic() {
        let compressed = CompressedPathNested::from_scan(vec![
            ScanFileTypeWithPath::File {
                path: PathBuf::from("/head/content"),
            },
            ScanFileTypeWithPath::File {
                path: PathBuf::from("/nope/test"),
            },
            ScanFileTypeWithPath::File {
                path: PathBuf::from("/head/other"),
            },
        ])
        .pretty_unwrap();
        // panic!("{compressed:?}");

        let mut iter = compressed.iter_path_vecs().collect::<Vec<_>>();
        assert!(iter.contains(&vec![OsStr::new("head"), OsStr::new("content")]));
        // assert_eq!(iter.next(), Some(Path::new("/nope/test").to_path_buf()));
        // assert_eq!(iter.next(), Some(Path::new("/head/other").to_path_buf()));
        // assert_eq!(iter.next(), None);
    }
}
