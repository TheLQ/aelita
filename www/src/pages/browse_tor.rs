use crate::controllers::handlebars::HbsPage;
use crate::controllers::state::WState;
use crate::err::WebResult;
use crate::server::util::BasicResponse;
use aelita_stor_diesel::{
    storapi_tor_torrents_list_starts_with, storapi_tor_torrents_list_starts_with_count,
};
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use std::collections::HashMap;
use xana_commons_rs::qbittorrent_re::serde_json;

pub async fn handle_browse_tor(
    State(state): State<WState<'_>>,
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

async fn render_search_json(state: WState<'_>, query: &str) -> WebResult<BasicResponse> {
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

    let json = serde_json::to_string(&children)?;
    Ok(BasicResponse(StatusCode::OK, mime::JSON, Body::from(json)))
}

async fn render_search_count(state: WState<'_>, query: &str) -> WebResult<BasicResponse> {
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

async fn render_html_list(state: WState<'_>) -> WebResult<BasicResponse> {
    // let tpl = get_template();
    // let body = tpl.render(())?;

    // let body = Body::from(get_html_const());

    state.render_page(HbsPage::Browse_Tor, ())
}
