use crate::controllers::state::WState;
use crate::err::WebResult;
use crate::pages::browse_journal::handle_browse_journal;
use crate::pages::browse_paths::{handle_browse_paths, handle_browse_paths_root};
use crate::pages::browse_tor::handle_browse_tor;
use crate::pages::fallback::handle_fallback;
use crate::pages::handle_root::handle_root;
use crate::pages::xrn_path::handle_xrn_path;
use crate::pages::xrn_space::handle_xrn_space;
use aelita_commons::log_init;
use aelita_stor_diesel::PermaStore;
use axum::Router;
use axum::http::header::CACHE_CONTROL;
use axum::http::{HeaderValue, Request};
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{MakeSpan, TraceLayer};
use xana_commons_rs::tracing_re::{Level, info};

/// Begin magic
pub async fn start_server() -> WebResult<()> {
    log_init();

    let sqlstate = WState::new(PermaStore::AelitaNull)?;

    let app = Router::new()
        .route("/", get(handle_root))
        .route("/browse/tor", get(handle_browse_tor))
        .route("/browse/journal", get(handle_browse_journal))
        .route("/browse/paths", get(handle_browse_paths_root))
        .route("/browse/paths{*path_raw}", get(handle_browse_paths))
        // xrn handling
        // route by prefix for performance
        // XrnFromUrl extractor parses this
        .route("/xrn:project{*xrn_value}", get(handle_xrn_space))
        .route("/xrn:path{*xrn_value}", get(handle_xrn_path))
        .nest_service("/scripts", ServeDir::new("www/scripts"))
        .fallback(handle_fallback)
        .with_state(sqlstate)
        // .layer(TraceLayer::new_for_http().make_span_with(SpanFactory {}))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http().make_span_with(SpanFactory {}))
                .layer(SetResponseHeaderLayer::overriding(
                    CACHE_CONTROL,
                    HeaderValue::from_static("no-cache"),
                )),
        );

    // run our app with hyper, listening globally on port 3000
    let addr = "0.0.0.0:4000";
    info!("Starting server on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
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
