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
    ///
    PlanningLabel
}