use aelita_commons::err_utils::pretty_error;
use aelita_commons::logs::{LOCALE, log_init_trace};
use aelita_commons::num_format_re::ToFormattedString;
use aelita_commons::tracing_re::{error, info};
use aelita_stor_diesel::api::api_journal::{
    NewMutation, storapi_journal_id_counter_get_or_init, storapi_journal_mutation_push,
    storapi_journal_reset_all,
};
use aelita_stor_diesel::api::common::with_counter;
use aelita_stor_diesel::connection::{StorConnection, establish_connection};
use aelita_stor_diesel::err::{StorDieselError, StorDieselResult};
use aelita_stor_diesel::models::model_journal::ModelJournalIdKey;
use diesel::MysqlConnection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{ExitCode, ExitStatus};
use std::time::Instant;
use std::{fs, io};

fn main() -> ExitCode {
    log_init_trace();

    if let Err(e) = inner_main() {
        error!("FAIL {}", pretty_error(e));
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HistoryItem {
    id: String,
    url: String,
    title: Option<String>,
    last_visit_time: usize,
    visit_count: u16,
}

fn inner_main() -> StorDieselResult<()> {
    let contents = load_file()?;

    let conn = &mut establish_connection();
    storapi_journal_reset_all(conn)?;

    insert_data(
        conn,
        vec![
            HistoryItem {
                id: "somefire".into(),
                last_visit_time: 9999,
                title: Some("sometitle".into()),
                url: "http://your-done.com".into(),
                visit_count: 1,
            },
            HistoryItem {
                id: "morefire".into(),
                last_visit_time: 9999,
                title: Some("moretitle".into()),
                url: "http://more-done.com".into(),
                visit_count: 1,
            },
        ],
    )?;

    insert_data(
        conn,
        vec![HistoryItem {
            id: "extrsafire".into(),
            last_visit_time: 9999,
            title: Some("extratitle".into()),
            url: "http://your-done.com".into(),
            visit_count: 1,
        }],
    )?;

    Ok(())
}

fn load_file() -> StorDieselResult<Vec<HistoryItem>> {
    let input_path = Path::new("../../../browser_history_desktop_2025-02-01.json");
    let ioec = StorDieselError::ioec(&input_path);

    let contents = fs::read_to_string(input_path).map_err(ioec.io())?;
    let history: Vec<HistoryItem> = serde_json::from_str(&contents).map_err(ioec.serde())?;
    let reconverted = serde_json::to_string(&history).map_err(ioec.serde())?;

    info!(
        "raw size          {}",
        reconverted.len().to_formatted_string(&LOCALE)
    );

    let mut previous = Vec::new();
    for i in 1..=22 {
        let start = Instant::now();
        let compressed = zstd::bulk::compress(reconverted.as_bytes(), i).unwrap();
        let cur_len = compressed.len();
        let wait = (Instant::now() - start).as_secs();

        let increase = previous
            .iter()
            .rev()
            .map(|p| format!("{:.1}", (cur_len as f32 / *p as f32) * 100.0))
            .take(5)
            .collect::<Vec<String>>()
            .join(",");
        let from_early = previous
            .get(2)
            .map(|p| format!("{:.1}", (cur_len as f32 / *p as f32) * 100.0))
            .unwrap_or("".into());
        info!(
            "compressed {i:02} size {} in {} secs - {} | {}",
            cur_len.to_formatted_string(&LOCALE),
            wait,
            increase,
            from_early
        );
        previous.push(cur_len);
    }

    todo!()

    // info!("Decoded {} entries", json.len());

    // Ok(json)
}

fn insert_data(
    conn: &mut StorConnection,
    data: impl IntoIterator<Item = HistoryItem>,
) -> StorDieselResult<()> {
    let counter = storapi_journal_id_counter_get_or_init(conn, ModelJournalIdKey::FireHistory)?;
    let mut cur_counter = counter.counter;

    storapi_journal_mutation_push(
        conn,
        data.into_iter()
            .map(with_counter(&mut cur_counter, |i, cur| NewMutation {
                mut_type: "asd".into(),
                data: "asd".into(),
            })),
    )?;

    Ok(())
}
