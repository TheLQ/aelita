use crate::controllers::state::WState;
use crate::err::WebResult;
use aelita_stor_diesel::{StorIdType, storapi_hd_get_path_by_path, storapi_hd_list_children_by_id};
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};

/// Difference from xrn_paths: We don't require a tree_id
pub async fn handle_browse_paths_root() -> Redirect {
    Redirect::to("/xrn:path:fs/")
}

pub async fn handle_browse_paths(
    State(state): State<WState>,
    Path(path_raw): Path<String>,
) -> WebResult<impl IntoResponse> {
    if path_raw.is_empty() || path_raw == "/" {
        return Ok(handle_browse_paths_root().await);
    }

    let path = std::path::Path::new(&path_raw).to_path_buf();
    let path_by_ids = state
        .sqlfs
        .transact({
            let path = path.clone();
            move |conn| storapi_hd_get_path_by_path(conn, &path)
        })
        .await?;

    Ok(Redirect::to(&format!(
        "/xrn:path:fs{path_raw}/__tree{}",
        path_by_ids.last().unwrap()
    )))
}
