use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::{WebError, WebResult};
use crate::server::convert_xrn::XrnFromUrl;
use crate::server::util::pretty_basic_page;
use aelita_stor_diesel::StorDieselError;
use aelita_stor_diesel::api_hd::storapi_hd_list_children;
use aelita_xrn::defs::path_xrn::PathXrn;
use axum::body::Body;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::Response;
use mime::TEXT_HTML;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::LazyLock;

pub async fn handle_xrn_path(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<PathXrn>,
) -> WebResult<Response> {
    _handle_xrn_path(state, xrn).await
}

async fn _handle_xrn_path(state: SqlState, xrn: PathXrn) -> WebResult<Response> {
    let children = state
        .sqlfs
        .transact({
            let xrn = xrn.clone();
            move |conn| {
                //
                storapi_hd_list_children(conn, xrn.path())
            }
        })
        .await;
    match children {
        Err(WebError::StorDiesel(StorDieselError::UnknownComponent(values, _))) => {
            Response::builder()
                .header(header::CONTENT_TYPE, TEXT_HTML.to_string())
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(pretty_basic_page(
                    "404 Path component(s) not found",
                    values,
                )))
                .map_err(Into::into)
        }
        Err(e) => Err(e)?,
        Ok(children) => Response::builder()
            .header(header::CONTENT_TYPE, TEXT_HTML.to_string())
            .status(StatusCode::OK)
            .body(render_html(xrn.path().to_path_buf(), children)?)
            .map_err(Into::into),
    }
}

fn render_html(root: PathBuf, children: Vec<String>) -> WebResult<Body> {
    #[derive(Serialize)]
    struct PathEntry {
        xrn: PathXrn,
        name: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        children: Vec<PathEntry>,
        root_title: String,
        parent_xrn: Option<PathXrn>,
    }
    let is_root_xrn = root.to_str().unwrap() == "/";
    let props = HtmlProps {
        root_title: root.to_str().unwrap().to_string(),
        parent_xrn: if is_root_xrn {
            None
        } else {
            Some(PathXrn::from_path(root.parent().unwrap()))
        },
        children: children
            .into_iter()
            .map(|name| PathEntry {
                xrn: PathXrn::from_path(root.join(&name)),
                name,
            })
            .collect(),
    };
    let tpl = get_template();
    tpl.render(props)
}

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/xrn_path.hbs");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
