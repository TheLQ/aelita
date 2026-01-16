use crate::change::change_hd::{HdAddPath, HdAddRoot, HdAddSymlink};
use crate::{HdAddPathToSpace, ModelJournalId, StorDieselResult, StorTransaction};
use serde::{Deserialize, Serialize};

pub trait Changer {
    type Result;

    fn commit_change(
        self,
        conn: &mut StorTransaction,
        change_context: ChangeContext,
    ) -> StorDieselResult<Self::Result>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChangeOp {
    HdAddPath(HdAddPath),
    HdAddSymlink(HdAddSymlink),
    HdAddRoot(HdAddRoot),
    HdAddPathToSpace(HdAddPathToSpace),
}

impl Changer for ChangeOp {
    type Result = ();

    fn commit_change(self, conn: &mut StorTransaction, ctx: ChangeContext) -> StorDieselResult<()> {
        match self {
            Self::HdAddPath(v) => v.commit_change(conn, ctx),
            Self::HdAddRoot(v) => v.commit_change(conn, ctx).map(|_| ()),
            Self::HdAddSymlink(v) => v.commit_change(conn, ctx),
            Self::HdAddPathToSpace(v) => v.commit_change(conn, ctx),
        }
    }
}

pub struct ChangeContext {
    pub journal_id: ModelJournalId,
}

#[cfg(test)]
mod test {
    use crate::ModelHdRoot;
    use crate::change::change_hd::{HdAddRoot, HdAddSymlink};
    use crate::change::defs::ChangeOp;

    #[test]
    fn enc_test() {
        let values = [
            ChangeOp::HdAddSymlink(HdAddSymlink {
                target: vec!["asdf".as_bytes().to_vec()],
                at: vec!["yep".as_bytes().to_vec()],
            }),
            ChangeOp::HdAddRoot(HdAddRoot {
                source: vec!["huh".as_bytes().to_vec()],
                description: "".to_string(),
                space_name: "".to_string(),
                root_type: ModelHdRoot::ZfsDataset,
            }),
        ];
        let out = serde_json::to_string(&values).unwrap();
        panic!("{out}")
    }
}
