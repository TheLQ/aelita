use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
use crate::server::convert_xrn::XrnFromUrl;
use aelita_stor_diesel::models::projects_model::ModelProject;
use aelita_xrn::defs::project_xrn::ProjectXrn;
use axum::Form;
use axum::body::Body;
use axum::extract::State;
use serde::Serialize;
use std::sync::LazyLock;

pub async fn handle_project(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<ProjectXrn>,
) -> WebResult<Body> {
    render_html(state, xrn).await
}

#[axum::debug_handler]
pub async fn handle_project_post(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<ProjectXrn>,
    Form(form): Form<ModelProject>,
) -> WebResult<Body> {
    state.sqlfs.project_names_push(vec![form]).await?;

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
                published: format!("{}", extract.published),
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
