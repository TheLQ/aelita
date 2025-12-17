use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
use crate::server::convert_xrn::XrnFromUrl;
use aelita_stor_diesel::api_hd::storapi_hd_list_children;
use aelita_xrn::defs::path_xrn::PathXrn;
use aelita_xrn::defs::space_xrn::SpaceXrn;
use axum::body::Body;
use axum::extract::State;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::LazyLock;

pub async fn handle_xrn_path(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<PathXrn>,
) -> WebResult<Body> {
    _handle_xrn_path(state, xrn).await
}

async fn _handle_xrn_path(state: SqlState, xrn: PathXrn) -> WebResult<Body> {
    let children = state
        .sqlfs
        .transact({
            let xrn = xrn.clone();
            move |conn| {
                //
                storapi_hd_list_children(conn, xrn.path())
            }
        })
        .await?;
    render_html(xrn.path().to_path_buf(), children)
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
    const TEMPLATE: &str = include_str!("../../html/xrn_paths.hbs");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
