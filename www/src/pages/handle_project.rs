use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
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

    let project = ModelProject {
        xrn: ProjectXrn::new(ProjectTypeXrn::from_str(&project_type)?, id.parse()?),
        title,
        published,
    };

    state.sqlfs.project_names_push(vec![project]).await?;

    render_html(state, xrn).await
}

async fn render_html(state: SqlState, xrn: ProjectXrn) -> WebResult<Body> {
    let query = state.sqlfs.project_names().await?;

    #[derive(Serialize)]
    struct ProjectEntry {
        xrn: String,
        published: String,
        title: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        projects: Vec<ProjectEntry>,
    }
    let props = HtmlProps {
        projects: query
            .into_iter()
            .map(|extract| ProjectEntry {
                xrn: extract.xrn.to_string(),
                published: extract.published.to_stor_string(),
                title: extract.title,
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
