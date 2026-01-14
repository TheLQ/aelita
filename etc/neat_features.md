(writing practice for non-technical readers. Not intended as a user guide)

# Aelita Journal

## Overview

To change, eg `thing` color to red, instead of changing `thing` directly we write that change to the journal.
To commit we read it back from the journal as a command to change the real `thing` color.

From an archival perspective we gain audibility and recovery.
tracing a value to the source instead of praying it wasn't from a bug.
If it was a bug we can't always fix without a complex restore from backup if they exist.
With a journal simply fix and re-commit the change. The resulting database is fixed.

With the ability to restore from the very first journal entry
you gain the interesting ability to radically redesign the database or data layer
then test by re-importing. Few common data storage systems have this,
though expensive "transactional" database companies exist now that help automate this if you follow their design.

By restoring from zero we also have the interesting ability to radically redesign then relatively simply rebuild from
the journal,
instead of complex multi-stage SQL migrations.
Also we scan disks with 10,000,000s of files every month and can trace history with a single id
instead of per-path change logs.

https://file.garden/aQjexz18lk1YUJs9/audio/ghostbusters.png

## Core Implementation

The journal is basically SQL BLOBs with id, type, date, etc metadata.
Generic so it can be used for any data byte Vec/array. For example some are JSON others are `postcard`-created binary

```sql
CREATE TABLE IF NOT EXISTS `journal_immutable`
(
    -- @formatter:off for massive enum
    `journal_id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    `journal_type`      ENUM ('Space1', 'QbGetTorJson1', 'NData1') NOT NULL,
    `metadata`          JSON,
    `committed`         BOOLEAN          NOT NULL,
    `at`                TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `cause_xrn`         VARCHAR(100),
    `cause_description` TEXT             NOT NULL,
    `data_hash`         BINARY(32),
    PRIMARY KEY (`journal_id`)
    -- @formatter:on
);

CREATE TABLE IF NOT EXISTS `journal_immutable_data`
(
    `journal_id` INTEGER UNSIGNED NOT NULL,
    `data`       LONGBLOB         NOT NULL,
    `data_id`    INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    PRIMARY KEY (`data_id`)
);
```

Currently there are

* Full filesystem scans as CompressedPaths, see section
* Qbittorrent torrents fetch as
* ChangeOp

## ChangeOp

After the big scanning 100,000s of values Journal types come the simple "just change `thing` color" style changes.

This simply moves all "change" function params to a serializable struct that implements `Changer` trait.
ChangeOp enum contains all possible changes. This is what's written to the database.
Then later fetched and used as parameters to the `Changer` that actually changes the database with our new value.

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum ChangeOp {
    HdAddPath(HdAddPath),
    HdAddSymlink(HdAddSymlink),
    HdAddRoot(HdAddRoot),
}

pub struct HdAddSymlink {
    pub at: Vec<Vec<u8>>,
    pub target: Vec<Vec<u8>>,
}

pub struct HdAddRoot {
    pub source: Vec<Vec<u8>>,
    pub description: String,
    pub space_name: String,
    pub root_type: ModelHdRoot,
}

impl Changer for HdAddRoot {
    ...
}
...
```

# CompressedPaths

## Overview

ndata (name data) is a list of filesystem paths for XanaCorp Aelita. The initial implementation stored it as a plain
list.
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
We create a separate Builder structure that can use any complex utility needed, then convert to the above simple
structure.

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