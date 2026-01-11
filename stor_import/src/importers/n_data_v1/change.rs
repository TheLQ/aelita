use crate::importers::change_op_v1::changer::Changer;
use aelita_stor_diesel::{StorDieselResult, StorTransaction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HdPathAddSymlink {
    pub source: Vec<Vec<u8>>,
    pub target: Vec<Vec<u8>>,
}
impl Changer for HdPathAddSymlink {
    fn commit_change(self, conn: &mut StorTransaction) -> StorDieselResult<()> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HdPathAddRoot {
    pub source: Vec<Vec<u8>>,
    pub desc: String,
}
impl Changer for HdPathAddRoot {
    fn commit_change(self, conn: &mut StorTransaction) -> StorDieselResult<()> {
        todo!()
    }
}
