use crate::controllers::handlebars::HbsPage;
use crate::controllers::state::WState;
use crate::err::{WebErrorCause, WebErrorKind, WebResult};
use crate::server::convert_xrn::XrnFromUrl;
use crate::server::util::{BasicResponse, pretty_basic_page};
use aelita_stor_diesel::err::StorDieselErrorKind;
use aelita_stor_diesel::{
    ModelFileTreeId, PathRow, storapi_hd_get_path_by_id, storapi_hd_list_children_by_id,
};
use aelita_xrn::defs::address::XrnAddr;
use aelita_xrn::defs::path_xrn::{PathXrn, PathXrnType};
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use std::path::{Component, PathBuf};
use xana_commons_rs::CrashErrKind;

pub async fn handle_xrn_path(
    State(state): State<WState>,
    XrnFromUrl(xrn): XrnFromUrl<PathXrn>,
) -> WebResult<BasicResponse> {
    _handle_xrn_path(state, xrn).await
}

async fn _handle_xrn_path(state: WState, xrn: PathXrn) -> WebResult<BasicResponse> {
    let tree_id = ModelFileTreeId::from_xrn(&xrn);
    let xrn_path = xrn.path().to_path_buf();
    let children_raw = state
        .sqlfs
        .transact({
            move |conn| {
                let (path_rows, db_path) = storapi_hd_get_path_by_id(conn, tree_id)?;
                let children = storapi_hd_list_children_by_id(conn, tree_id)?;
                Ok((path_rows, db_path, children))
            }
        })
        .await;

    if let Err(e) = &children_raw
        && let Some(cause) = &e.xana_err().cause
        && let WebErrorCause::StorDieselError(StorDieselErrorKind::UnknownComponent) = cause
    {
        return Ok(BasicResponse(
            StatusCode::OK,
            mime::HTML,
            Body::from(pretty_basic_page("404 Path component(s) not found", xrn)),
        ));
    }
    let (path_rows, db_path, children) = children_raw?;
    if db_path != xrn_path {
        return Err(WebErrorKind::PathXrnNotEqualDatabase.build_message(format!(
            "input {} database {}",
            xrn_path.display(),
            db_path.display()
        )));
    }

    render_html(state, xrn, path_rows, children)
}

fn render_html(
    state: WState,
    xrn: PathXrn,
    path_rows: Vec<PathRow>,
    children: Vec<PathRow>,
) -> WebResult<BasicResponse> {
    let mut breadcrumbs = Vec::new();
    let path = xrn.path();
    for (i, row) in path_rows.iter().enumerate() {
        let mut path_iter = path.components();
        assert_eq!(path_iter.next(), Some(Component::RootDir));
        let partial_path: PathBuf = ["/"]
            .into_iter()
            .chain(
                path_rows
                    .iter()
                    .take(i)
                    .map(|v| str::from_utf8(&v.component).unwrap()),
            )
            .collect();

        breadcrumbs.push(PathXrn::new(
            PathXrnType::Fs,
            partial_path,
            row.association.tree_id,
        ))
    }

    #[derive(Serialize)]
    struct PathEntry {
        xrn: XrnAddr,
        name: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        children: Vec<PathEntry>,
        root_title: String,
        breadcrumbs: Vec<XrnAddr>,
    }
    let props = HtmlProps {
        root_title: xrn.to_string(),
        breadcrumbs,
        children: children
            .into_iter()
            .map(|row| {
                let comp_name = str::from_utf8(&row.component).unwrap();
                PathEntry {
                    xrn: PathXrn::new(
                        PathXrnType::Fs,
                        path.join(comp_name),
                        row.association.tree_id,
                    ),
                    name: comp_name.to_string(),
                }
            })
            .collect(),
    };
    state.render_page(HbsPage::Xrn_Path, props)
}
