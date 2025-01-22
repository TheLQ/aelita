use crate::controllers::sqlcontroller::SqlState;
use axum::extract::{Path, State};
use itertools::Itertools;

pub async fn handle_xrns(State(state): State<SqlState>, Path(xrn): Path<String>) -> String {
    let res = state.sqlfs.xrns_list().await.unwrap();

    let extraction = res.into_iter().map(|v| format!("{:?}", v)).join("'");
    let url_xrn = xrn;
    format!("{extraction}\n{url_xrn}\n")
}
