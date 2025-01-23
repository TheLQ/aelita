use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
use crate::server::convert_xrn::XrnFromUrl;
use aelita_commons::tracing_re::{info, warn};
use aelita_xrn::defs::address::XrnAddr;
use aelita_xrn::defs::project_xrn::ProjectXrn;
use axum::body::Body;
use axum::debug_handler;
use axum::extract::{Path, State};

#[debug_handler]
pub async fn handle_project(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<ProjectXrn>,
) -> WebResult<Body> {
    render_html(state, xrn)
}

fn render_html(state: SqlState, xrn: ProjectXrn) -> WebResult<Body> {
    Ok(Body::from(format!("project xrn {:?}", xrn)))
}
