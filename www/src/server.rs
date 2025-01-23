use crate::controllers::sqlcontroller::SqlState;
use crate::pages::handle_registry::{handle_xrns_html, handle_xrns_html_post, handle_xrns_root};
use crate::pages::handle_root::handle_root;
use aelita_commons::logs::log_init_trace;
use axum::Router;
use axum::routing::{get, post};

/// Begin magic
#[tokio::main]
pub async fn start_server() {
    log_init_trace();

    let sqlstate = SqlState::new();

    let app = Router::new()
        .route("/", get(handle_root))
        .route("/{xrn}", get(handle_xrns_root))
        .route("/{xrn}/html", get(handle_xrns_html))
        .route("/{xrn}/html", post(handle_xrns_html_post))
        .with_state(sqlstate);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
