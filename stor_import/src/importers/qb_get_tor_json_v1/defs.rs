use aelita_stor_diesel::id_types::ModelQbHostId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ImportQbMetadata {
    pub qb_host_id: ModelQbHostId,
}
