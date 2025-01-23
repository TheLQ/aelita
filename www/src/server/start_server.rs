use crate::controllers::sqlcontroller::SqlState;
use crate::pages::handle_project::handle_project_html;
use crate::pages::handle_registry::{
    handle_registry_html_post, handle_registry_root, handle_registryt_html,
};
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
        .route("/registry", get(handle_registry_root))
        .route("/registry/html", get(handle_registryt_html))
        .route("/registry/html", post(handle_registry_html_post))
        .route("/xrn:project:{xrn_value}/html", get(handle_project_html))
        .with_state(sqlstate);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
