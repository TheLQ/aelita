use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
use crate::server::convert_xrn::XrnFromUrl;
use aelita_xrn::defs::address::XrnAddr;
use axum::body::Body;
use axum::debug_handler;
use axum::extract::State;

#[debug_handler]
pub async fn handle_project_html(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl,
) -> WebResult<Body> {
    render_html(state, xrn)
}

fn render_html(state: SqlState, xrn: XrnAddr) -> WebResult<Body> {
    Ok(Body::from(format!("project xrn {:?}", xrn)))
}
