use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::{SqlState, basic_cause};
use crate::err::WebResult;
use aelita_stor_diesel::api_journal::{storapi_journal_get_created, storapi_journal_list};
use aelita_xrn::defs::address::XrnAddr;
use axum::Form;
use axum::body::Body;
use axum::extract::{Path, State};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::LazyLock;

pub async fn handle_xrn_journal(
    State(state): State<SqlState>,
    xrn_raw: Option<Path<String>>,
) -> WebResult<Body> {
    todo!()
}

// #[derive(Deserialize)]
// pub struct PagePost {
//     xrn_name: String,
// }

// pub async fn handle_registry_html_post(
//     State(state): State<SqlState>,
//     Form(form): Form<PagePost>,
// ) -> WebResult<Body> {
//     let new = vec![ModelRegistryId {
//         xrn: XrnAddr::from_str(&form.xrn_name)?.to_string(),
//         published: StorDate::now(),
//         publish_cause: basic_cause("frontend-form"),
//     }];
//     state
//         .sqlfs
//         .query_stor(|conn| storapi_registry_ids_push(conn, new))
//         .await?;
//
//     // show same page
//     handle_registry_html(State(state), None).await
// }

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/xrn_journal.hbs");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
