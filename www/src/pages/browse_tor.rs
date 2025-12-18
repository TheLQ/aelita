use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
use crate::server::util::BasicResponse;
use aelita_stor_diesel::api_tor::{
    storapi_tor_torrents_list_starts_with, storapi_tor_torrents_list_starts_with_count,
};
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use xana_commons_rs::qbittorrent_re::serde_json;

pub async fn handle_browse_tor(
    State(state): State<SqlState>,
    Query(params): Query<HashMap<String, String>>,
) -> WebResult<BasicResponse> {
    if let Some(query) = params.get("query") {
        if let Some(_) = params.get("as_count") {
            render_search_count(state, query.as_str()).await
        } else {
            render_search_json(state, query.as_str()).await
        }
    } else {
        render_html_list(state).await
    }
}

async fn render_search_json(state: SqlState, query: &str) -> WebResult<BasicResponse> {
    let children = state
        .sqlfs
        .transact({
            let query = query.to_string();
            move |conn| {
                //
                storapi_tor_torrents_list_starts_with(conn, &query)
            }
        })
        .await?;

    #[derive(Serialize)]
    struct TorEntry {
        name: String,
        status: String,
        path: String,
    }
    let front_children = children
        .into_iter()
        .map(|tor| TorEntry {
            name: tor.name.clone(),
            status: tor.state.inner().to_string(),
            path: tor.name.to_string(),
        })
        .collect::<Vec<_>>();

    let json = serde_json::to_string(&front_children)?;
    Ok(BasicResponse(StatusCode::OK, mime::JSON, Body::from(json)))
}

async fn render_search_count(state: SqlState, query: &str) -> WebResult<BasicResponse> {
    let children = state
        .sqlfs
        .transact({
            let query = query.to_string();
            move |conn| {
                //
                storapi_tor_torrents_list_starts_with_count(conn, &query)
            }
        })
        .await?;
    Ok(BasicResponse(
        StatusCode::OK,
        mime::PLAIN,
        Body::from(children.to_string()),
    ))
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
