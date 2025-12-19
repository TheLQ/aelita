use aelita_www::server::start_server::start_server;
use std::process::ExitCode;
use xana_commons_rs::{pretty_main, pretty_main_async};

#[tokio::main]
async fn main() -> ExitCode {
    pretty_main_async(start_server).await
}
