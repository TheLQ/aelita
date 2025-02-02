use aelita_commons::err_utils::pretty_error;
use aelita_commons::logs::log_init_trace;
use aelita_commons::tracing_re::{error, info};
use aelita_stor_diesel::err::{StorDieselError, StorDieselResult};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::{ExitCode, ExitStatus};

fn main() -> ExitCode {
    log_init_trace();

    if let Err(e) = inner_main() {
        error!("FAIL {}", pretty_error(e));
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn inner_main() -> StorDieselResult<()> {
    let contents = load_file()?;

    Ok(())
}

fn load_file() -> StorDieselResult<Vec<HistoryItem>> {
    let input_path = Path::new("./browser_history_desktop_2025-02-01.json");
    let ioec = StorDieselError::ioec(input_path.to_path_buf());

    let contents = fs::read_to_string(input_path).map_err(ioec.io())?;
    let json: Vec<HistoryItem> = serde_json::from_str(&contents).map_err(ioec.serde())?;
    info!("Decoded {} entries", json.len());

    Ok(json)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HistoryItem {
    id: String,
    url: String,
    title: Option<String>,
    last_visit_time: usize,
    visit_count: u16,
}
