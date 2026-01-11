use crate::err::StorImportResult;
use crate::importers::n_data_v1::change::{HdAddPath, HdAddRoot, HdAddSymlink};
use aelita_stor_diesel::StorTransaction;
use serde::{Deserialize, Serialize};

pub trait Changer {
    fn commit_change(self, conn: &mut StorTransaction) -> StorImportResult<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) enum ChangeOp {
    HdAddPath(HdAddPath),
    HdAddSymlink(HdAddSymlink),
    HdAddRoot(HdAddRoot),
}

impl Changer for ChangeOp {
    fn commit_change(self, conn: &mut StorTransaction) -> StorImportResult<()> {
        match self {
            Self::HdAddPath(v) => v.commit_change(conn),
            Self::HdAddRoot(v) => v.commit_change(conn),
            Self::HdAddSymlink(v) => v.commit_change(conn),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::importers::change_op_v1::changer::ChangeOp;
    use crate::importers::n_data_v1::change::{HdAddRoot, HdAddSymlink};
    use aelita_stor_diesel::ModelHdRoot;

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
