use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
use aelita_stor_diesel::api_journal::storapi_journal_list;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};

pub async fn handle_browse_paths_root(State(state): State<SqlState>) -> impl IntoResponse {
    Redirect::to("/browse/paths/")
}

pub async fn handle_browse_paths(
    State(state): State<SqlState>,
    Path(path_raw): Path<String>,
) -> WebResult<Body> {
    let journals = state
        .sqlfs
        .transact(|conn| storapi_journal_list(conn))
        .await?;

    let path = std::path::Path::new(&path_raw);
    Ok(Body::from(path.to_str().unwrap().to_string()))
}
