use crate::err::StorDieselErrorKind;
use crate::{ModelFileCompId, ModelLocalTreeId, StorDieselResult, StorIdType};
use indexmap::IndexSet;
use itertools::Itertools;
use rayon::prelude::ParallelSliceMut;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::stdin;
use std::iter::Peekable;
use std::ops::DerefMut;
use std::os::unix::prelude::OsStrExt;
use std::path::{Component, Path, PathBuf};
use xana_commons_rs::tracing_re::{info, trace, warn};
use xana_commons_rs::{
    CommaJoiner, CrashErrKind, ProgressWidget, ScanFileType, ScanFileTypeWithPath, ScanStat,
};

/// Store file tree as a... tree.
/// Because Vec<PathBuf> is very inefficient at 10,000,000s of files
/// Optimized for small serialized size
#[derive(Debug, Serialize, Deserialize)]
pub struct CompressedPaths {
    parts: Vec<Vec<u8>>,
    nodes: Vec<CompNode>,
}

impl CompressedPaths {
    fn from_scan_builder(
        mut scans: Vec<(ScanFileTypeWithPath, ScanStat)>,
    ) -> CompressedPathBuilder {
        info!("starting sort");
        scans.par_sort_by_cached_key(|v| v.0.path().clone());

        // warn!("Load complete. press enter to continue...");
        // let mut _in = String::new();
        // stdin()
        //     .read_line(&mut _in)
        //     .expect("Did not enter a correct string");

        let mut build = CompressedPathBuilder::new();
        let total_scans = scans.len();
        let mut progress = ProgressWidget::new(4096 * 4);
        for (i, (scan, stat)) in scans.into_iter().enumerate() {
            progress.log(i, total_scans, |msg| info!("scan import {msg}"));

            let (path, stype) = scan.into_parts();
            // info!("scanning {}", path.display());
            build.push_path(&path, stype, stat)
        }
        build
    }

    pub fn parts(&self) -> &[Vec<u8>] {
        self.parts.as_slice()
    }

    pub fn nodes(&self) -> &[CompNode] {
        self.nodes.as_slice()
    }

    pub fn node_id(&self, node_id: ModelLocalTreeId) -> &CompNode {
        &self.nodes[node_id.inner_usize()]
    }

    pub fn node_children_unwrap(&self, node_id: ModelLocalTreeId) -> &Vec<ModelLocalTreeId> {
        let CompNodeType::Dir { children_node_ids } = &self.node_id(node_id).node_type else {
            panic!("not a dir at node {node_id}")
        };
        children_node_ids
    }

    pub fn from_scan(scans: Vec<(ScanFileTypeWithPath, ScanStat)>) -> StorDieselResult<Self> {
        Self::from_build(Self::from_scan_builder(scans))
    }

    fn from_build(builder: CompressedPathBuilder) -> StorDieselResult<Self> {
        let nodes_res = (0..builder.nodes.len())
            .into_par_iter()
            .map(|i| CompNode::from_builder(&builder, ModelLocalTreeId::new_usize(i)))
            .collect::<Vec<_>>();
        let nodes = nodes_res.into_iter().try_collect()?;

        // let nodes = (0..builder.nodes.len())
        //     .into_iter()
        //     .map(|i| CompNode::from_builder(&builder, ModelLocalTreeId::new_usize(i)))
        //     .try_collect()?;
        Ok(Self {
            parts: builder
                .parts
                .into_iter()
                .map(|v| v.as_bytes().to_vec())
                .collect(),
            nodes,
        })
    }

    pub fn iter_parent_child<'i>(&'i self) -> CompressedIter<'i> {
        CompressedIter::new(self)
    }

    // todo generic between builder and main
    fn path_vec_from_node_id(&self, node_id: ModelLocalTreeId) -> Vec<&[u8]> {
        let mut path_rev = Vec::new();
        let mut next_id = node_id;
        loop {
            let cur_node = &self.node_id(next_id);
            path_rev.push(self.parts[cur_node.name_comp_id].as_slice());
            next_id = cur_node.parent;
            if next_id.inner_id() == 0 {
                break;
            }
        }
        path_rev.reverse();
        path_rev
    }

    pub fn debug_log(&self) {
        for (i, part) in self.parts.iter().enumerate() {
            trace!("part {i} | - {}", str::from_utf8(part).unwrap());
        }
        for (i, node) in self.nodes.iter().enumerate() {
            trace!("part {i} | - {node:?}");
        }
    }
}

/// Optimized for cheap modifying with extra lookup fields
struct CompressedPathBuilder {
    parts: IndexSet<OsString>,
    nodes: Vec<CompNodeBuilder>,
    /// By pre-sorting the input paths we can cache eg 9/10 components
    /// Vastly improving performance with 30 million paths up to 9 levels deep
    /// 10k/sec to 250k/sec
    cache: Vec<CachedLookup>,
    fast_path: PathBuf,
}

impl CompressedPathBuilder {
    fn new() -> Self {
        Self {
            parts: IndexSet::new(),
            nodes: vec![CompNodeBuilder::new_comp_id(
                usize::MAX - 100,
                ModelLocalTreeId::new(0),
            )],
            cache: Vec::new(),
            fast_path: PathBuf::from("/"),
        }
    }

    fn node_id(&self, node_id: ModelLocalTreeId) -> &CompNodeBuilder {
        self.nodes.get(node_id.inner_usize()).unwrap()
    }

    fn node_id_mut(&mut self, node_id: ModelLocalTreeId) -> &mut CompNodeBuilder {
        self.nodes.get_mut(node_id.inner_usize()).unwrap()
    }

    fn push_path(&mut self, path: &Path, new_node_type: ScanFileType, stat: ScanStat) {
        // trace!("pushing {}", path.display());

        // fast short circuit
        let mut last_index;
        if self.fast_path.as_os_str() != "/" {
            loop {
                // [Path::starts_with] expensively compares each .components()
                // Instead compare byte slices
                let path_bytes = path.as_os_str().as_bytes();
                let fast_bytes = self.fast_path.as_os_str().as_bytes();
                if path_bytes.starts_with(fast_bytes)
                    // avoid a/bb from matching a/bbbb
                    && path_bytes[fast_bytes.len()] == b'/'
                {
                    break;
                } else {
                    let is_root = !self.fast_path.pop();
                    if is_root {
                        break;
                    }
                }
            }

            let cached_comps = self.fast_path.iter().skip(1).count();
            if cached_comps < 2 {
                warn!(
                    "fast path {} useless for {}",
                    self.fast_path.display(),
                    path.display()
                );
            } else if cached_comps > 50 {
                panic!(
                    "fast path {} useless for {}",
                    self.fast_path.display(),
                    path.display()
                );
            }
            self.cache.truncate(cached_comps);

            if let Some(last_cache) = self.cache.last() {
                // assert_eq!(last_cache.component, self.fast_path.iter().last().unwrap());
                last_index = last_cache.child_id;
            } else {
                // empty, reset
                last_index = ModelLocalTreeId::new(0);
            }
        } else {
            last_index = ModelLocalTreeId::new(0);
        }

        let remain_path = {
            let path_bytes = path.as_os_str().as_bytes();
            let fast_bytes = self.fast_path.as_os_str().as_bytes();
            // trace!(
            //     "fast {} remove from {}",
            //     self.fast_path.display(),
            //     path.display()
            // );
            if path_bytes == b"/" {
                path
            } else if fast_bytes == b"/" {
                Path::new(OsStr::from_bytes(&path_bytes[1..]))
            } else {
                Path::new(OsStr::from_bytes(
                    &path_bytes[(fast_bytes.len() + /*slash*/1)..],
                ))
            }
        };
        // trace!("remain {}", remain_path.display());
        for comp in remain_path.components() {
            let Component::Normal(comp) = comp else {
                panic!("invalid path {}", path.display())
            };

            let comp_index = self.parts.get_index_of(comp).unwrap_or_else(|| {
                self.parts.insert(comp.to_os_string());
                self.parts.len() - 1
            });

            if let Some(node_id) = self.find_node_children(last_index, comp_index) {
                last_index = node_id;
            } else {
                let next_index = ModelLocalTreeId::new_usize(self.nodes.len());
                let prev_node = self.node_id_mut(last_index);
                prev_node.children_indexes.push(next_index);
                prev_node.children_comp_ids.push(comp_index);
                self.nodes
                    .push(CompNodeBuilder::new_comp_id(comp_index, last_index));
                last_index = next_index;
            };

            self.cache.push(CachedLookup {
                child_id: last_index,
                component: comp.to_os_string(),
            })
        }

        if let Some(parent) = path.parent() {
            self.fast_path = parent.to_path_buf();
        }

        let last = self.node_id_mut(last_index);
        match new_node_type {
            ScanFileType::Dir => {
                last.node_type = Some(CompNodeType::Dir {
                    children_node_ids: Vec::new(),
                })
            }
            ScanFileType::File => last.node_type = Some(CompNodeType::File),
            ScanFileType::Symlink { target } => last.delayed_symlink = Some(target),
        };
        last.stat = Some(stat);
    }

    fn find_node_children(
        &self,
        node_id: ModelLocalTreeId,
        needle_comp_id: usize,
    ) -> Option<ModelLocalTreeId> {
        let node = &self.nodes[node_id.inner_usize()];
        // [slice::contains] for usize is niche optimized
        if node.children_comp_ids.contains(&needle_comp_id) {
            let res = node
                .children_comp_ids
                .iter()
                .position(|v| *v == needle_comp_id)?;
            Some(node.children_indexes[res])
        } else {
            None
        }
    }

    // todo generic between builder and main
    fn path_vec_from_node_id(&self, node_id: ModelLocalTreeId) -> Vec<&OsString> {
        let mut path_rev = Vec::new();
        let mut next_id = node_id;
        while next_id.inner_id() != 0 {
            let cur_node = self.node_id(next_id);
            path_rev.push(&self.parts[cur_node.name_comp_id]);
            next_id = cur_node.parent;
            if next_id.inner_id() == 0 {
                break;
            }
        }
        path_rev.reverse();
        path_rev
    }

    fn pathbuf_from_node_id(&self, node_id: ModelLocalTreeId) -> PathBuf {
        let res = PathBuf::from("/");
        res.join(PathBuf::from_iter(
            self.path_vec_from_node_id(node_id).into_iter(),
        ))
    }

    fn path_to_node_id(&self, path: &Path) -> Option<ModelLocalTreeId> {
        let mut comps = path.components();
        assert_eq!(comps.next(), Some(Component::RootDir));
        let mut next_id = ModelLocalTreeId::new(0);
        for comp in comps {
            let Component::Normal(comp) = comp else {
                return None;
            };
            let Some(comp_id) = self.parts.get_index_of(comp) else {
                return None;
            };
            let Some(child_id) = self.find_node_children(next_id, comp_id) else {
                return None;
            };
            next_id = child_id;
        }
        Some(next_id)
    }
}

struct CompNodeBuilder {
    parent: ModelLocalTreeId,
    name_comp_id: usize,
    node_type: Option<CompNodeType>,
    children_indexes: Vec<ModelLocalTreeId>,
    children_comp_ids: Vec<usize>,
    delayed_symlink: Option<PathBuf>,
    stat: Option<ScanStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompNode {
    parent: ModelLocalTreeId,
    name_comp_id: usize,
    node_type: CompNodeType,
    stat: Option<ScanStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompNodeType {
    Dir {
        children_node_ids: Vec<ModelLocalTreeId>,
    },
    File,
    Symlink {
        target_node_id: ModelLocalTreeId,
    },
    BrokenSymlink {
        raw: Vec<u8>,
    },
}

impl CompNodeBuilder {
    fn new_comp_id(name_comp_id: usize, parent: ModelLocalTreeId) -> Self {
        Self {
            parent,
            name_comp_id,
            node_type: None,
            children_indexes: Vec::new(),
            children_comp_ids: Vec::new(),
            delayed_symlink: None,
            stat: None,
        }
    }
}

impl CompNode {
    fn from_builder(
        compressed_builder: &CompressedPathBuilder,
        node_id: ModelLocalTreeId,
        // debug_context: &mut PathBuf,
    ) -> StorDieselResult<Self> {
        let CompNodeBuilder {
            parent,
            name_comp_id,
            node_type,
            children_indexes,
            delayed_symlink,
            stat,
            children_comp_ids,
        } = compressed_builder.node_id(node_id);
        let mut symlink_type = None;
        // debug_context.push(&name);
        if let Some(delayed_symlink) = delayed_symlink {
            assert!(node_type.is_none(), "{node_type:?}");
            if delayed_symlink.is_absolute() {
                let mut comps = delayed_symlink.components();
                assert_eq!(comps.next(), Some(Component::RootDir));

                let mut last_index = Some(ModelLocalTreeId::new(0));
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
                    symlink_type = Some(CompNodeType::Symlink { target_node_id })
                } else {
                    symlink_type = Some(CompNodeType::BrokenSymlink {
                        raw: delayed_symlink.as_os_str().as_bytes().to_vec(),
                    });
                }
            } else {
                let cur_path: PathBuf = compressed_builder.pathbuf_from_node_id(node_id);
                let new_path_raw = cur_path.join(delayed_symlink);
                let new_path = new_path_raw.normalize_lexically().map_err(|_marker_e| {
                    StorDieselErrorKind::SymlinkResolveFailed.build_message(new_path_raw.display())
                })?;
                if let Some(target_node_id) = compressed_builder.path_to_node_id(&new_path) {
                    symlink_type = Some(CompNodeType::Symlink { target_node_id })
                } else {
                    symlink_type = Some(CompNodeType::BrokenSymlink {
                        raw: delayed_symlink.as_os_str().as_bytes().to_vec(),
                    });
                }
                // this path should now resolve
            }
        }
        // if debug_context != Path::new("/") {
        //     assert!(debug_context.pop(), "cur {}", debug_context.display());
        // }
        let res_node_type = symlink_type
            .or_else(|| match node_type {
                // was created empty
                Some(CompNodeType::Dir { .. }) => Some(CompNodeType::Dir {
                    children_node_ids: children_indexes.clone(),
                }),
                None => None,
                Some(v) => {
                    assert!(
                        children_indexes.is_empty(),
                        "node {node_id} type {v:?} but has _ children",
                        // children_indexes
                        //     .iter()
                        //     .map(|v| v.to_string())
                        //     .collect::<CommaJoiner>()
                    );
                    Some(v.clone())
                }
            })
            .unwrap_or_else(|| {
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
            });
        if stat.is_none() {
            warn!("stat missing for {node_id}")
        }

        Ok(Self {
            parent: *parent,
            name_comp_id: *name_comp_id,
            node_type: res_node_type,
            stat: stat.clone(),
        })
    }

    pub fn node_type(&self) -> &CompNodeType {
        &self.node_type
    }

    pub fn name_from<'c>(&self, compressed: &'c CompressedPaths) -> &'c [u8] {
        compressed.parts[self.name_comp_id].as_slice()
    }
}
struct CachedLookup {
    component: OsString,
    child_id: ModelLocalTreeId,
}

pub struct CompressedIter<'c> {
    backend: &'c CompressedPaths,
    cursor_stack: Vec<IterStack>,
}

impl<'c> CompressedIter<'c> {
    fn new(backend: &'c CompressedPaths) -> Self {
        Self {
            backend,
            cursor_stack: vec![IterStack {
                node_id: ModelLocalTreeId::new(0),
                next_child: 0,
            }],
        }
    }
}

impl<'c> Iterator for CompressedIter<'c> {
    type Item = IterEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(IterStack {
            node_id,
            next_child,
        }) = self.cursor_stack.last().cloned()
        else {
            return None;
        };

        let children_node_ids = self.backend.node_children_unwrap(node_id);
        if next_child >= children_node_ids.len() {
            self.cursor_stack.pop();
            trace!("pop {node_id} after EOF");
            return self.next();
        } else {
            let last = self.cursor_stack.last_mut().unwrap();
            last.next_child += 1;
        }

        let child_id = children_node_ids[next_child];
        let cur_entry = IterEntry {
            node_id: child_id,
            parent_id: node_id,
        };

        match &self.backend.node_id(child_id).node_type {
            CompNodeType::Dir { .. } => {
                trace!("descend into {child_id}");
                self.cursor_stack.push(IterStack {
                    node_id: child_id,
                    next_child: 0,
                });
                Some(cur_entry)
            }
            CompNodeType::File => Some(cur_entry),
            CompNodeType::Symlink { target_node_id } => {
                trace!("ignore unsupported symlink {target_node_id}");
                self.next()
            }
            CompNodeType::BrokenSymlink { raw } => {
                trace!(
                    "ignore unsupported broken symlink {}",
                    str::from_utf8(raw).unwrap_or("NOT_UTF8")
                );
                self.next()
            }
        }
    }
}

#[derive(Clone)]
pub struct IterStack {
    pub node_id: ModelLocalTreeId,
    pub next_child: usize,
}

#[derive(PartialEq, Debug)]
pub struct IterEntry {
    pub parent_id: ModelLocalTreeId,
    pub node_id: ModelLocalTreeId,
}

pub struct IterFileEntry<'c> {
    node: &'c CompNode,
    parent_node: &'c CompNode,
}

#[cfg(test)]
mod test {
    use super::{CompressedPathBuilder, CompressedPaths};
    use crate::{ModelLocalTreeId, StorIdType};
    use aelita_commons::log_init;
    use std::ffi::OsStr;
    use std::path::PathBuf;
    use xana_commons_rs::tracing_re::info;
    use xana_commons_rs::{PrettyUnwrap, ScanFileTypeWithPath, ScanStat};

    #[test]
    fn basic() {
        log_init();

        let compressed = easy_compressed();

        compressed.debug_log();

        let iter = compressed.iter_parent_child().collect::<Vec<_>>();
        for entry in iter {
            info!("{:?}", entry);
        }

        // assert!(iter.contains(&vec![b"head".into(), b"content".into()]),);
        // assert!(iter.contains(&vec![b"nope", b"test"]),);
        // assert!(iter.contains(&vec![b"head", b"other"]),);
    }

    #[test]
    fn get_path() {
        log_init();
        let compressed = easy_compressed_builder();
        for i in 0..compressed.nodes.len() {}

        assert_eq!(
            compressed.path_vec_from_node_id(ModelLocalTreeId::new(3)),
            vec![
                OsStr::new("head"),
                OsStr::new("content"),
                OsStr::new("other")
            ]
        );
    }

    #[test]
    fn path() {
        let mut start = PathBuf::from("/a/b/c/d");
        for i in 0..9 {
            start.pop();
            // println!("{}", start.display());
            for v in start.iter() {
                print!("{} - ", v.to_str().unwrap());
            }
            println!()
        }
    }

    fn easy_compressed_builder() -> CompressedPathBuilder {
        CompressedPaths::from_scan_builder(vec![
            (
                ScanFileTypeWithPath::Dir {
                    path: PathBuf::from("/head"),
                },
                ScanStat::dummy_value(),
            ),
            (
                ScanFileTypeWithPath::Dir {
                    path: PathBuf::from("/head/content"),
                },
                ScanStat::dummy_value(),
            ),
            (
                ScanFileTypeWithPath::File {
                    path: PathBuf::from("/head/content/other"),
                },
                ScanStat::dummy_value(),
            ),
            (
                ScanFileTypeWithPath::File {
                    path: PathBuf::from("/head/content/nest"),
                },
                ScanStat::dummy_value(),
            ),
            (
                ScanFileTypeWithPath::File {
                    path: PathBuf::from("/head/content/sub"),
                },
                ScanStat::dummy_value(),
            ),
            (
                ScanFileTypeWithPath::File {
                    path: PathBuf::from("/nope/test"),
                },
                ScanStat::dummy_value(),
            ),
            (
                ScanFileTypeWithPath::File {
                    path: PathBuf::from("/head/deep/other"),
                },
                ScanStat::dummy_value(),
            ),
            (
                ScanFileTypeWithPath::File {
                    path: PathBuf::from("/head/deep/more"),
                },
                ScanStat::dummy_value(),
            ),
            (
                ScanFileTypeWithPath::File {
                    path: PathBuf::from("/head/deep/and"),
                },
                ScanStat::dummy_value(),
            ),
        ])
    }

    fn easy_compressed() -> CompressedPaths {
        CompressedPaths::from_build(easy_compressed_builder()).pretty_unwrap()
    }
}
