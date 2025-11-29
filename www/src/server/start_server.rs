use crate::controllers::sqlcontroller::SqlState;
use crate::log::www_log_init;
use crate::pages::fallback::handle_fallback;
use crate::pages::handle_project::{handle_project, handle_project_post};
use crate::pages::handle_registry::{
    handle_registry_html, handle_registry_html_post, handle_registry_root,
};
use crate::pages::handle_root::handle_root;
use axum::Router;
use axum::http::Request;
use axum::routing::{get, post};
use tower_http::trace::{MakeSpan, TraceLayer};
use xana_commons_rs::tracing_re::Level;

/// Begin magic
#[tokio::main]
pub async fn start_server() {
    www_log_init();

    let sqlstate = SqlState::new();

    let app = Router::new()
        .route("/", get(handle_root))
        .route("/registry", get(handle_registry_root))
        .route("/registry/html", get(handle_registry_html))
        .route("/registry/html/{xrn}", get(handle_registry_html))
        .route("/registry/html", post(handle_registry_html_post))
        // xrn handling
        // route by prefix for performance
        // XrnFromUrl extractor parses this
        .route("/xrn:project:{*xrn_value}", get(handle_project))
        .route("/xrn:project:{*xrn_value}", post(handle_project_post))
        .fallback(handle_fallback)
        .with_state(sqlstate)
        .layer(TraceLayer::new_for_http().make_span_with(SpanFactory {}));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
struct SpanFactory {}

impl<B> MakeSpan<B> for SpanFactory {
    fn make_span(&mut self, request: &Request<B>) -> xana_commons_rs::tracing_re::Span {
        xana_commons_rs::tracing_re::span!(
            Level::DEBUG,
            "request",
            method = %request.method(),
            uri = %request.uri(),
            // headers = ?request.headers(),
        )
    }
}
