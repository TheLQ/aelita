use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
use crate::pages::handle_registry::PagePost;
use crate::server::convert_xrn::XrnFromUrl;
use aelita_commons::tracing_re::{info, warn};
use aelita_stor_diesel::diesel_re::insertable::DefaultableColumnInsertValue::Default;
use aelita_xrn::defs::address::XrnAddr;
use aelita_xrn::defs::project_xrn::{ProjectTypeXrn, ProjectXrn};
use axum::body::Body;
use axum::extract::{Path, State};
use axum::{Form, debug_handler};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub async fn handle_project(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<ProjectXrn>,
) -> WebResult<Body> {
    render_html(state, xrn).await
}

#[derive(Deserialize)]
pub struct ProjectPost {
    ptype: ProjectTypeXrn,
    id: u64,
}

pub async fn handle_project_post(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<ProjectXrn>,
    Form(form): Form<ProjectPost>,
) -> WebResult<Body> {
    render_html(state, xrn).await
}

async fn render_html(state: SqlState, xrn: ProjectXrn) -> WebResult<Body> {
    let query = state.sqlfs.projects_list().await?;

    #[derive(Serialize)]
    struct ProjectEntry {
        xrn: String,
        published: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        projects: Vec<ProjectEntry>,
    }
    let props = HtmlProps {
        projects: query
            .into_iter()
            .map(|extract| ProjectEntry {
                xrn: extract.xrn,
                published: format!("{}", extract.published),
            })
            .collect(),
    };
    let tpl = get_template();
    tpl.render(props)
}

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/page_projects.html");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
