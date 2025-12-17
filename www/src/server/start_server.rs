use crate::controllers::sqlcontroller::SqlState;
use crate::pages::browse_journal::handle_browse_journal;
use crate::pages::browse_paths::{handle_browse_paths, handle_browse_paths_root};
use crate::pages::fallback::handle_fallback;
use crate::pages::handle_root::handle_root;
use crate::pages::xrn_path::handle_xrn_path;
use crate::pages::xrn_space::handle_xrn_space;
use aelita_commons::log_init;
use aelita_stor_diesel::PermaStore;
use axum::Router;
use axum::http::Request;
use axum::routing::{get, post};
use tower_http::trace::{MakeSpan, TraceLayer};
use xana_commons_rs::tracing_re::{Level, info};

pub const CSS_HTML: &str =
    "<link rel='stylesheet' href='https://cdn.jsdelivr.net/npm/bulma@1.0.4/css/bulma.min.css'>";

/// Begin magic
#[tokio::main]
pub async fn start_server() {
    log_init();

    let sqlstate = SqlState::new(PermaStore::AelitaNull);

    let app = Router::new()
        .route("/", get(handle_root))
        .route("/browse/journal", get(handle_browse_journal))
        .route("/browse/paths", get(handle_browse_paths_root))
        .route("/browse/paths{*path_raw}", get(handle_browse_paths))
        // xrn handling
        // route by prefix for performance
        // XrnFromUrl extractor parses this
        .route("/xrn:project{*xrn_value}", get(handle_xrn_space))
        .route("/xrn:path{*xrn_value}", get(handle_xrn_path))
        .fallback(handle_fallback)
        .with_state(sqlstate)
        .layer(TraceLayer::new_for_http().make_span_with(SpanFactory {}));

    // run our app with hyper, listening globally on port 3000
    let addr = "0.0.0.0:4000";
    info!("Starting server on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
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
