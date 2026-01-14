(writing practice for non-technical readers. Not intended as a user guide)

# CompressedPaths

## Overview

Rdata (rpool data) is a list of filesystem paths for XWS Aelita. The initial implementation stored it as a plain list.
But that is very inefficient compared to a tree structure. The list takes over 5gb while in a tree structure is 1gb (
-76%).

The goal is compress

```text
/a/b/some
/a/b/other
/a/more
/mnt/backups/old-pc/var/lib/archives/pc-2024/c/Documents and Settings/me/My Documents/My Games/Borderlands 2/WillowGame/SaveData/save.dat
/mnt/backups/old-pc/var/lib/archives/pc-2024/c/Documents and Settings/me/My Documents/My Games/Borderlands 2/Config/Config.ini
```

into

![todo](https://upload.wikimedia.org/wikipedia/commons/thumb/5/5e/Binary_tree_v2.svg/500px-Binary_tree_v2.svg.png)

Not in scope

* Advanced query functions: Intended to simply be the Rust container to insert into the database. Query the database
  instead.

## Implementation

The tree design is a basic single vec of nodes with parent ids. This closely matches database table hd1_files_parents.
Database also has tree_depth but isn't nessesary for our needs, will create on insert.

Path components are stored separately in a parts Vec. Each component is in raw bytes form.
This matches database table hd1_files_components.

```rust
#[derive(Serialize, Deserialize)]
pub struct CompressedPaths {
    parts: Vec<Vec<u8>>,
    nodes: Vec<CompNode>,
}

struct CompNode {
    parent: FsNodeId,
    name_comp_id: FsCompId,
    node_type: CompNodeType,
    stat: ScanStat,
}
```

Storing effectively as the database does is the smallest possible structure. It's the output of the complex builder
below.
This is then serialized to a data file with postcard's binary encoding, then compressed with zstd.

```text
output from compressed paths efficency log
```

We need to convert 30 million (currently) paths into this structure.
In Rust it's too slow as-is because we are missing the SQL indexes and query engine.
We create a Builder structure that can use any complex utility needed, then convert to the above simple structure.

This design changes nodes to "double-linked": parent knows their children and children know their parent.  
If given '/a/b/c', I can find 'c' in 'b' searching eg <10 files (only some files in that folder)
instead of every file and folder that exists.

Another optimization is pre-sorting the input paths and using a cache.
Say we are 10 levels deep /a/b/c/d/e/f/g/h/i/j and the next file is j2,
we can cache every path id up to i instead of expensively searching every parent-child for every deep path.
This gave a 5x+ improvement from 10-50k/sec peak to 250k-500k/sec minimum,
and reducing commit time from 30 to 10 minutes saving 10s of dev hours per month.

The goal is every operation (get, insert) searches the least amount of nodes possible.

```rust
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

struct CompNodeBuilder {
    parent: FsNodeId,
    name_comp_id: FsCompId,
    node_type: Option<CompNodeType>,
    children_indexes: Vec<FsNodeId>,
    children_comp_ids: Vec<usize>,
    delayed_symlink: Option<PathBuf>,
    stat: Option<ScanStat>,
}

struct CachedLookup {
    component: OsString,
    child_id: FsNodeId,
}
```
