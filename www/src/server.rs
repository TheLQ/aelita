use crate::controllers::sqlcontroller::{SqlController, SqlState};
use crate::pages::handle_root::handle_root;
use crate::pages::handle_xrns::handle_xrns;
use aelita_commons::logs::log_init_trace;
use aelita_stor_diesel::schema::xrn_registry::dsl::xrn_registry;
use axum::Router;
use axum::routing::get;

#[tokio::main]
pub async fn start_server() {
    log_init_trace();

    let sqlstate = SqlState::new();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(handle_root))
        .route("/{xrn}", get(handle_xrns))
        .with_state(sqlstate);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
