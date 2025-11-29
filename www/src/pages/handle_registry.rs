use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::{SqlState, basic_cause};
use crate::err::WebResult;
use aelita_stor_diesel::api::api_registry_ids::{
    storapi_registry_ids_list, storapi_registry_ids_push,
};
use aelita_stor_diesel::models::ModelRegistryId;
use aelita_stor_diesel::models::date::StorDate;
use aelita_xrn::defs::address::XrnAddr;
use axum::Form;
use axum::body::Body;
use axum::extract::{Path, State};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::LazyLock;

pub async fn handle_registry_root(
    State(state): State<SqlState>,
    Path(xrn): Path<String>,
) -> String {
    let query = state
        .sqlfs
        .query_stor(storapi_registry_ids_list)
        .await
        .unwrap();

    let extraction = query.into_iter().map(|v| format!("{:?}", v)).join("'");
    let url_xrn = xrn;
    format!("{extraction}\n{url_xrn}\n")
}

async fn render_html_list(state: SqlState) -> WebResult<Body> {
    let query = state.sqlfs.query_stor(storapi_registry_ids_list).await?;
    #[derive(Serialize)]
    struct XrnEntry {
        xrn: String,
        published: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        xrns: Vec<XrnEntry>,
    }
    let props = HtmlProps {
        xrns: query
            .into_iter()
            .map(|extract| XrnEntry {
                xrn: extract.xrn.to_string(),
                published: extract.published.to_string(),
            })
            .collect(),
    };
    let tpl = get_template();
    tpl.render(props)
}

async fn render_html_detail(state: SqlState, xrn: XrnAddr) -> WebResult<Body> {
    todo!()
}

pub async fn handle_registry_html(
    State(state): State<SqlState>,
    xrn_raw: Option<Path<String>>,
) -> WebResult<Body> {
    if let Some(Path(xrn)) = xrn_raw {
        render_html_detail(state, XrnAddr::from_str(&xrn)?).await
    } else {
        render_html_list(state).await
    }
}

#[derive(Deserialize)]
pub struct PagePost {
    xrn_name: String,
}

pub async fn handle_registry_html_post(
    State(state): State<SqlState>,
    Form(form): Form<PagePost>,
) -> WebResult<Body> {
    let new = vec![ModelRegistryId {
        xrn: XrnAddr::from_str(&form.xrn_name)?.to_string(),
        published: StorDate::now(),
        publish_cause: basic_cause("frontend-form"),
    }];
    state
        .sqlfs
        .query_stor(|conn| storapi_registry_ids_push(conn, new))
        .await?;

    // show same page
    handle_registry_html(State(state), None).await
}

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/page_xrns.html.hbs");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
