use crate::importers::change_op_v1::changer::Changer;
use aelita_stor_diesel::{
    DisplayCompPath, ModelHdRoot, ModelJournalId, NewHdRoot, NewModelSpaceName, StorDieselResult,
    StorIdTypeDiesel, StorTransaction, storapi_hd_links_add, storapi_hdroots_push,
};
use serde::{Deserialize, Serialize};
use xana_commons_rs::tracing_re::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct HdAddSymlink {
    pub at: Vec<Vec<u8>>,
    pub target: Vec<Vec<u8>>,
}
impl Changer for HdAddSymlink {
    fn commit_change(self, conn: &mut StorTransaction) -> StorDieselResult<()> {
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
    fn commit_change(self, conn: &mut StorTransaction) -> StorDieselResult<()> {
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
                journal_id: ModelJournalId::invalid_value(),
                description,
                space_name,
            },
            NewHdRoot { rtype: root_type },
        )?;
        Ok(())
    }
}
