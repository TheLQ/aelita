use crate::importers::n_data_v1::change::{HdPathAddRoot, HdPathAddSymlink};
use aelita_stor_diesel::{StorDieselResult, StorTransaction};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait Changer {
    fn commit_change(self, conn: &mut StorTransaction) -> StorDieselResult<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) enum ChangeOp {
    HdPathAddSymlink(HdPathAddSymlink),
    HdPathAddRoot(HdPathAddRoot),
}

impl Changer for ChangeOp {
    fn commit_change(self, conn: &mut StorTransaction) -> StorDieselResult<()> {
        match self {
            Self::HdPathAddRoot(v) => v.commit_change(conn),
            Self::HdPathAddSymlink(v) => v.commit_change(conn),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::importers::change_op_v1::changer::ChangeOp;
    use crate::importers::n_data_v1::change::{HdPathAddRoot, HdPathAddSymlink};

    #[test]
    fn enc_test() {
        let values = [
            ChangeOp::HdPathAddSymlink(HdPathAddSymlink {
                target: vec!["asdf".as_bytes().to_vec()],
                source: vec!["yep".as_bytes().to_vec()],
            }),
            ChangeOp::HdPathAddRoot(HdPathAddRoot {
                source: vec!["huh".as_bytes().to_vec()],
                desc: "yasdf".into(),
            }),
        ];
        let out = serde_json::to_string(&values).unwrap();
        panic!("{out}")
    }
}
