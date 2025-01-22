use axum::Router;
use axum::routing::get;
use aelita_commons::logs::log_init_trace;
use crate::pages::root::handle_root;

#[tokio::main]
pub async fn start_server() {
    log_init_trace();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(handle_root));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}