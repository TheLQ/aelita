/// xrn:project:1000000
pub struct XrnAddr {
    atype: XrnAddrType,
    value: String,
}

pub enum XrnAddrType {
    /// A working project
    Project,
    /// A physically stored file
    A3,
    /// For displaying entities, this is a rating
    PlanningLabel,
    /// Syncs data from other sources to here
    SyncJob,
}