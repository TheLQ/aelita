use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::{WebError, WebResult};
use crate::server::util::BasicResponse;
use aelita_stor_diesel::api_hd::storapi_hd_list_children;
use aelita_stor_diesel::api_journal::storapi_journal_list;
use aelita_stor_diesel::api_tor::{
    storapi_tor_torrents_list_starts_with, storapi_tor_torrents_update_status_batch,
};
use aelita_stor_diesel::model_journal::ModelJournalImmutableDiesel;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::Response;
use serde::Serialize;
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::sync::LazyLock;
use xana_commons_rs::qbittorrent_re::serde_json;

pub async fn handle_browse_tor(
    State(state): State<SqlState>,
    Query(params): Query<HashMap<String, String>>,
) -> WebResult<BasicResponse> {
    if let Some(prefix) = params.get("prefix") {
        render_prefix_search_json(state, prefix.as_str()).await
    } else {
        render_html_list(state).await
    }
}

async fn render_prefix_search_json(state: SqlState, prefix: &str) -> WebResult<BasicResponse> {
    let children = state
        .sqlfs
        .transact({
            let prefix = prefix.to_string();
            move |conn| {
                //
                storapi_tor_torrents_list_starts_with(conn, &prefix)
            }
        })
        .await?;

    #[derive(Serialize)]
    struct TorEntry {
        name: String,
    }
    let front_children = children
        .into_iter()
        .map(|tor| TorEntry { name: tor.name })
        .collect::<Vec<_>>();

    let json = serde_json::to_string(&front_children)?;
    Ok(BasicResponse(StatusCode::OK, mime::JSON, Body::from(json)))
}

async fn render_html_list(state: SqlState) -> WebResult<BasicResponse> {
    // let tpl = get_template();
    // let body = tpl.render(())?;
    let body = Body::from(get_html());
    Ok(BasicResponse(StatusCode::OK, mime::HTML, body))
}

fn get_html() -> &'static str {
    const TEMPLATE: &str = include_str!("../../html/browse_tor.html");
    &TEMPLATE
}

// fn get_template() -> &'static HandlebarsPage {
//     const TEMPLATE: &str = include_str!("../../html/browse_tor.hbs");
//     static INSTANCE: LazyLock<HandlebarsPage> =
//         LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
//     &INSTANCE
// }
