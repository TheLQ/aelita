use xana_commons_rs::{MapHugeCrateName, XanaCommonsLogConfig};

pub fn log_init() {
    XanaCommonsLogConfig::<AelitaLog>::new_map_huge().log_init_trace()
}

struct AelitaLog;
impl MapHugeCrateName for AelitaLog {
    fn map_huge(huge_crate_name: &str) -> Option<&'static str> {
        match huge_crate_name {
            "aelita_stor_import" => Some("stor_import"),
            "aelita_stor_diesel" => Some("stor_diesel"),
            _ => None,
        }
    }
}
