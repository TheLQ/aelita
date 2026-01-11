use crate::change::defs::Changer;
use crate::err::StorDieselErrorKind;
use crate::{
    DisplayCompPath, ModelHdRoot, ModelJournalId, NewHdRoot, NewModelSpaceName, StorDieselResult,
    StorIdTypeDiesel, StorTransaction, storapi_hd_links_add, storapi_hd_tree_push,
    storapi_hdroots_push,
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
