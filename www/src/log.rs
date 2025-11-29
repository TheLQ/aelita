use xana_commons_rs::XanaCommonsLogConfig;

pub fn www_log_init() {
    XanaCommonsLogConfig::new_default()
        .with_extra_filter_env("tower_http=trace")
        .with_hide_tokio_thread_name(true)
        .log_init_trace();
}

// struct AelitaMapHuge;
// impl MapHugeCrateName for AelitaMapHuge {
//     fn map_huge(huge_crate_name: &str) -> Option<&'static str> {
//
//     }
// }
