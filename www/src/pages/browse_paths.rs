use crate::controllers::handlebars::HbsPage;
use crate::controllers::state::WState;
use crate::err::WebResult;
use crate::pages::base_html::BaseHtml;
use crate::server::util::BasicResponse;
use aelita_stor_diesel::{
    ModelFileTreeId, PathRow, StorIdType, storapi_hd_get_path_by_path,
    storapi_hd_list_children_by_id, storapi_hd_list_children_by_path,
};
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use serde::Serialize;
use std::path::PathBuf;

/// Difference from xrn_paths: We don't require a tree_id
pub async fn handle_browse_paths_root() -> impl IntoResponse {
    Redirect::to("/browse/paths/")
}

pub async fn handle_browse_paths(
    State(state): State<WState>,
    Path(path_raw): Path<String>,
) -> WebResult<BasicResponse> {
    let path = std::path::Path::new(&path_raw).to_path_buf();
    let (path_by_ids, children) = state
        .sqlfs
        .transact({
            let path = path.clone();
            move |conn| {
                let path_by_ids = storapi_hd_get_path_by_path(conn, &path_raw)?;
                let children = storapi_hd_list_children_by_id(conn, *path_by_ids.last().unwrap())?;
                Ok((path_by_ids, children))
                // storapi_hd_list_children_by_path(conn, path)
            }
        })
        .await?;

    render_html(state, path, path_by_ids, children)
}

fn render_html(
    state: WState,
    root: PathBuf,
    path_by_ids: Vec<ModelFileTreeId>,
    children: Vec<PathRow>,
) -> WebResult<BasicResponse> {
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
    let data = BaseHtml::title("Browse Tor").build(HtmlProps {
        root_path: root.to_str().unwrap().to_string(),
        children: children
            .into_iter()
            .map(|row| PathEntry {
                href: row.association.tree_id,
                name,
            })
            .collect(),
    });

    state.render_page(HbsPage::Browse_Paths, data)
}
