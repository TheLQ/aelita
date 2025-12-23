use crate::controllers::handlebars::HbsPage;
use crate::controllers::state::WState;
use crate::err::{WebError, WebErrorMeta, WebResult};
use crate::server::convert_xrn::XrnFromUrl;
use crate::server::util::{BasicResponse, pretty_basic_page};
use aelita_stor_diesel::err::StorDieselErrorKind;
use aelita_stor_diesel::storapi_hd_list_children;
use aelita_xrn::defs::path_xrn::PathXrn;
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use std::path::Path;

pub async fn handle_xrn_path(
    State(state): State<WState<'_>>,
    XrnFromUrl(xrn): XrnFromUrl<PathXrn>,
) -> WebResult<BasicResponse> {
    _handle_xrn_path(state, xrn).await
}

async fn _handle_xrn_path(state: WState<'_>, xrn: PathXrn) -> WebResult<BasicResponse> {
    let path = xrn.path();
    let path = if let Some(path) = path {
        path.to_path_buf()
    } else {
        todo!()
    };

    let path_clone = path.clone();
    let children = state
        .sqlfs
        .transact({
            move |conn| {
                //
                storapi_hd_list_children(conn, path_clone)
            }
        })
        .await;
    match children {
        Err(e)
            if matches!(
                *e,
                WebError {
                    meta: WebErrorMeta::StorDieselError(StorDieselErrorKind::UnknownComponent),
                    ..
                }
            ) =>
        {
            Ok(BasicResponse(
                StatusCode::OK,
                mime::HTML,
                Body::from(pretty_basic_page("404 Path component(s) not found", xrn)),
            ))
        }
        Err(e) => Err(e)?,
        // xrn.path().unwrap()
        Ok(children) => render_html(state, &path, children),
    }
}

fn render_html(state: WState<'_>, root: &Path, children: Vec<String>) -> WebResult<BasicResponse> {
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
