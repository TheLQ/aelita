use crate::controllers::handlebars::HbsPage;
use crate::controllers::state::WState;
use crate::err::{WebError, WebResult};
use crate::server::convert_xrn::XrnFromUrl;
use crate::server::util::{BasicResponse, pretty_basic_page};
use aelita_stor_diesel::StorDieselError;
use aelita_stor_diesel::storapi_hd_list_children;
use aelita_xrn::defs::path_xrn::PathXrn;
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use std::path::PathBuf;

pub async fn handle_xrn_path(
    State(state): State<WState<'_>>,
    XrnFromUrl(xrn): XrnFromUrl<PathXrn>,
) -> WebResult<BasicResponse> {
    _handle_xrn_path(state, xrn).await
}

async fn _handle_xrn_path(state: WState<'_>, xrn: PathXrn) -> WebResult<BasicResponse> {
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
            Ok(BasicResponse(
                StatusCode::OK,
                mime::HTML,
                Body::from(pretty_basic_page("404 Path component(s) not found", values)),
            ))
        }
        Err(e) => Err(e)?,
        Ok(children) => render_html(state, xrn.path().to_path_buf(), children),
    }
}

fn render_html(
    state: WState<'_>,
    root: PathBuf,
    children: Vec<String>,
) -> WebResult<BasicResponse> {
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
    state.render_page(HbsPage::Xrn_Path, props)
}
