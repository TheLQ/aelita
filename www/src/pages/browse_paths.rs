use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::{WebError, WebResult};
use aelita_stor_diesel::api_hd::storapi_hd_list_children;
use aelita_stor_diesel::api_journal::storapi_journal_list;
use aelita_stor_diesel::model_hd::HdPathDiesel;
use aelita_stor_diesel::model_journal::ModelJournalImmutableDiesel;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use handlebars::html_escape;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::LazyLock;

pub async fn handle_browse_paths_root(State(state): State<SqlState>) -> impl IntoResponse {
    Redirect::to("/browse/paths/")
}

pub async fn handle_browse_paths(
    State(state): State<SqlState>,
    Path(path_raw): Path<String>,
) -> WebResult<Body> {
    let path = std::path::Path::new(&path_raw).to_path_buf();
    let children = state
        .sqlfs
        .transact({
            let path = path.clone();
            move |conn| storapi_hd_list_children(conn, path)
        })
        .await?;

    render_html(path, children)
}

fn render_html(root: PathBuf, children: Vec<String>) -> WebResult<Body> {
    #[derive(Serialize)]
    struct PathEntry {
        href: String,
        name: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        children: Vec<PathEntry>,
        root_path: String,
    }
    let props = HtmlProps {
        root_path: root.to_str().unwrap().to_string(),
        children: children
            .into_iter()
            .map(|name| PathEntry {
                href: root.join(&name).to_str().unwrap().to_string(),
                name,
            })
            .collect(),
    };
    let tpl = get_template();
    tpl.render(props)
}

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/browse_paths.hbs");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
