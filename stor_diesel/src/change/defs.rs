use crate::change::change_hd::{HdAddPath, HdAddRoot, HdAddSymlink};
use crate::{ModelJournalId, StorDieselResult, StorTransaction};
use serde::{Deserialize, Serialize};

pub trait Changer {
    fn commit_change(
        self,
        conn: &mut StorTransaction,
        journal_id: ModelJournalId,
    ) -> StorDieselResult<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChangeOp {
    HdAddPath(HdAddPath),
    HdAddSymlink(HdAddSymlink),
    HdAddRoot(HdAddRoot),
}

impl Changer for ChangeOp {
    fn commit_change(
        self,
        conn: &mut StorTransaction,
        journal_id: ModelJournalId,
    ) -> StorDieselResult<()> {
        match self {
            Self::HdAddPath(v) => v.commit_change(conn, journal_id),
            Self::HdAddRoot(v) => v.commit_change(conn, journal_id),
            Self::HdAddSymlink(v) => v.commit_change(conn, journal_id),
        }
    }
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
