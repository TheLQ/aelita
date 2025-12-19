use aelita_stor_diesel::ModelQbHostId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportQbMetadata {
    pub qb_host_id: ModelQbHostId,
}
