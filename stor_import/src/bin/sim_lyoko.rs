use aelita_commons::log_init;
use std::process::ExitCode;
use xana_commons_rs::pretty_main;

fn main() -> ExitCode {
    log_init();
    pretty_main(aelita_stor_import::integ_test::sim_lyoko::sim_lyoko)
}
