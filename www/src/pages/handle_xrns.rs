use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use axum::extract::{Path, State};
use itertools::Itertools;
use std::sync::{LazyLock, OnceLock};

pub async fn handle_xrns(State(state): State<SqlState>, Path(xrn): Path<String>) -> String {
    let res = state.sqlfs.xrns_list().await.unwrap();

    let extraction = res.into_iter().map(|v| format!("{:?}", v)).join("'");
    let url_xrn = xrn;
    format!("{extraction}\n{url_xrn}\n")
}

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/page_xrns.html");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
