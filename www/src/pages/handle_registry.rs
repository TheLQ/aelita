use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
use aelita_stor_diesel::diesel_re::internal::derives::multiconnection::chrono::NaiveDateTime;
use aelita_stor_diesel::models::NewXrnExtraction;
use axum::Form;
use axum::body::Body;
use axum::extract::{Path, State};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub async fn handle_registry_root(
    State(state): State<SqlState>,
    Path(xrn): Path<String>,
) -> String {
    let res = state.sqlfs.xrns_list().await.unwrap();

    let extraction = res.into_iter().map(|v| format!("{:?}", v)).join("'");
    let url_xrn = xrn;
    format!("{extraction}\n{url_xrn}\n")
}

async fn render_html(state: SqlState, _xrn: String) -> WebResult<Body> {
    let query = state.sqlfs.xrns_list().await?;
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
                xrn: extract.xrn,
                published: format!("{}", extract.published),
            })
            .collect(),
    };
    let tpl = get_template();
    tpl.render(props)
}

pub async fn handle_registryt_html(
    State(state): State<SqlState>,
    Path(xrn): Path<String>,
) -> WebResult<Body> {
    render_html(state, xrn).await
}

#[derive(Deserialize)]
pub struct PagePost {
    xrn_name: String,
}

pub async fn handle_registry_html_post(
    State(state): State<SqlState>,
    Path(xrn): Path<String>,
    Form(form): Form<PagePost>,
) -> WebResult<Body> {
    let new = vec![NewXrnExtraction {
        xrn: form.xrn_name,
        // todo
        published: NaiveDateTime::default(),
    }];
    state.sqlfs.xrns_push(new).await?;

    // show same page
    render_html(state, xrn).await
}

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/page_xrns.html");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
