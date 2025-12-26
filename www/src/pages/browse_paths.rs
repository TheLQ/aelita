use crate::controllers::handlebars::HbsPage;
use crate::controllers::state::WState;
use crate::err::WebResult;
use crate::server::util::BasicResponse;
use aelita_stor_diesel::storapi_hd_list_children_by_path;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use serde::Serialize;
use std::path::PathBuf;

pub async fn handle_browse_paths_root() -> impl IntoResponse {
    Redirect::to("/browse/paths/")
}

pub async fn handle_browse_paths(
    State(state): State<WState>,
    Path(path_raw): Path<String>,
) -> WebResult<BasicResponse> {
    let path = std::path::Path::new(&path_raw).to_path_buf();
    let children = state
        .sqlfs
        .transact({
            let path = path.clone();
            move |conn| storapi_hd_list_children_by_path(conn, path)
        })
        .await?;

    render_html(state, path, children)
}

fn render_html(state: WState, root: PathBuf, children: Vec<String>) -> WebResult<BasicResponse> {
    #[derive(Serialize)]
    struct PathEntry {
        href: String,
        name: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        children: Vec<PathEntry>,
        root_path: String,
    }
    let props = HtmlProps {
        root_path: root.to_str().unwrap().to_string(),
        children: children
            .into_iter()
            .map(|name| PathEntry {
                href: root.join(&name).to_str().unwrap().to_string(),
                name,
            })
            .collect(),
    };
    state.render_page(HbsPage::Browse_Journal, props)
}
