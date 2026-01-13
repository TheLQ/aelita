use crate::change::defs::Changer;
use crate::err::StorDieselErrorKind;
use crate::{
    DisplayCompPath, ModelFileTreeId, ModelHdRoot, ModelJournalId, NewHdRoot, NewModelSpaceName,
    StorDieselResult, StorTransaction, convert_path_to_comps, storapi_hd_get_path_by_path,
    storapi_hd_links_add, storapi_hd_tree_push, storapi_hd_tree_push_single, storapi_hdroots_push,
};
use serde::{Deserialize, Serialize};
use xana_commons_rs::CrashErrKind;
use xana_commons_rs::tracing_re::info;
use xana_fs_indexer_rs::{CompressedPaths, ScanFileTypeWithPath, ScanStat};

#[derive(Debug, Serialize, Deserialize)]
pub struct HdAddPath {
    /// components * path * new_paths
    pub paths: Vec<(ScanFileTypeWithPath, ScanStat)>,
}
impl Changer for HdAddPath {
    fn commit_change(
        self,
        conn: &mut StorTransaction,
        _journal_id: ModelJournalId,
    ) -> StorDieselResult<()> {
        let Self { paths } = self;

        let preview = 5;

        if paths.is_empty() {
            panic!("no empty")
        } else if paths.len() < preview {
            for path in paths {
                commit_add_path_sql(conn, path)?;
            }
            Ok(())
        } else {
            for (scan_type, _stat) in paths.iter().take(preview) {
                info!("Add path {}", scan_type.path().display());
            }
            if paths.len() >= preview {
                info!("... to len {}", paths.len())
            }
            // todo: this is expensive for 1 path...
            let new_paths = CompressedPaths::from_scan(paths)
                .map_err(StorDieselErrorKind::InvalidChangeCompressedPaths.xana_map())?;
            storapi_hd_tree_push(conn, new_paths)
        }
    }
}

fn commit_add_path_sql(
    conn: &mut StorTransaction,
    (scan_type, stat): (ScanFileTypeWithPath, ScanStat),
) -> StorDieselResult<ModelFileTreeId> {
    let path = scan_type.path();
    let path_comps = convert_path_to_comps(path)?;
    let existing_ids = storapi_hd_get_path_by_path(conn, &path_comps)?;

    if path_comps.len() == existing_ids.len() {
        Err(StorDieselErrorKind::PathAlreadyExists.build_message(path.display()))
    } else if path_comps.len() - 1 == existing_ids.len() {
        // perfect, add a single batch
        let parent = existing_ids.last().unwrap();
        let file_comp = path_comps.last().unwrap();
        let file_id = storapi_hd_tree_push_single(conn, Some(*parent), &[(file_comp, stat)])?;
        // get stats
        Ok(file_id)
    } else {
        // todo adding more than once starts making the SQL equivalent of CompressedPaths
        Err(StorDieselErrorKind::PathFileParentMissing.build_message(path.display()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HdAddSymlink {
    pub at: Vec<Vec<u8>>,
    pub target: Vec<Vec<u8>>,
}
impl Changer for HdAddSymlink {
    fn commit_change(
        self,
        conn: &mut StorTransaction,
        _journal_id: ModelJournalId,
    ) -> StorDieselResult<()> {
        let Self { at, target } = self;
        info!(
            "Link {} to {}",
            DisplayCompPath(at.as_slice()),
            DisplayCompPath(target.as_slice())
        );
        storapi_hd_links_add(conn, at.as_slice(), target.as_slice())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HdAddRoot {
    pub source: Vec<Vec<u8>>,
    pub description: String,
    pub space_name: String,
    pub root_type: ModelHdRoot,
}
impl Changer for HdAddRoot {
    fn commit_change(
        self,
        conn: &mut StorTransaction,
        journal_id: ModelJournalId,
    ) -> StorDieselResult<()> {
        let Self {
            source,
            description,
            space_name,
            root_type,
        } = self;
        info!(
            "Add root {} name {space_name} ({description})",
            DisplayCompPath(source.as_slice()),
        );
        storapi_hdroots_push(
            conn,
            NewModelSpaceName {
                journal_id,
                description,
                space_name,
            },
            NewHdRoot { rtype: root_type },
        )?;
        Ok(())
    }
}
