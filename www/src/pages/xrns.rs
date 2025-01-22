use crate::controllers::sqlcontroller::SqlState;
use axum::extract::{Path, State};

pub async fn handle_xrns(State(state): State<SqlState>, Path(xrn): Path<String>) -> String {
    state.sqlfs.xrns_list();

    "Hello world".into()
}
