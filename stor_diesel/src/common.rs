use xana_commons_rs::XanaCommonsLogConfig;

pub fn log_init_trace() {
    XanaCommonsLogConfig::new_default().log_init_trace()
}
