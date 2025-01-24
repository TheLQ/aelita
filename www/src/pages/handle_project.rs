use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::{WebError, WebResult};
use crate::server::convert_xrn::XrnFromUrl;
use aelita_stor_diesel::date_wrapper::StorDate;
use aelita_stor_diesel::models::model_project::ModelProject;
use aelita_xrn::defs::project_xrn::{ProjectTypeXrn, ProjectXrn};
use axum::Form;
use axum::body::Body;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::LazyLock;

pub async fn handle_project(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<ProjectXrn>,
) -> WebResult<Body> {
    render_html(state, xrn).await
}

#[derive(Deserialize)]
pub struct PostData {
    pub project_type: String,
    pub id: String,
    pub title: String,
}

pub async fn handle_project_post(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<ProjectXrn>,
    Form(PostData {
        project_type,
        id,
        title,
    }): Form<PostData>,
) -> WebResult<Body> {
    let published: StorDate = StorDate::now();
    let xrn_project_id: u32 = id.parse().unwrap();

    let project = ModelProject {
        xrn_project_id,
        title,
        published,
    };

    state.sqlfs.project_names_push(vec![project]).await?;

    render_html(state, xrn).await
}

async fn render_html(state: SqlState, xrn: ProjectXrn) -> WebResult<Body> {
    match xrn.ptype() {
        ProjectTypeXrn::Dash => render_dash(state, xrn).await,
        ptype => Err(WebError::UnsupportedXrnRoute(ptype.as_ref().into())),
    }
}

async fn render_dash(state: SqlState, xrn: ProjectXrn) -> WebResult<Body> {
    match xrn.id() {
        0 => render_dash_primary(state, xrn).await,
        id => Err(WebError::UnsupportedDashboard(id)),
    }
}

async fn render_dash_primary(state: SqlState, xrn: ProjectXrn) -> WebResult<Body> {
    let query = state.sqlfs.project_names().await?;
    #[derive(Serialize)]
    struct ProjectEntry {
        xrn: String,
        published: String,
        title: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        dash_name: String,
        projects: Vec<ProjectEntry>,
    }
    let props = HtmlProps {
        dash_name: xrn.to_string(),
        projects: query
            .into_iter()
            .map(|extract| ProjectEntry {
                xrn: extract.xrn_project_id.to_string(),
                published: extract.published.to_stor_string(),
                title: extract.title,
            })
            .collect(),
    };
    let tpl = get_template();
    tpl.render(props)
}

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/page_projects.html.hbs");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
