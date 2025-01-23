use axum::extract::Request;

pub async fn handle_fallback(request: Request) -> String {
    let input_uri = request.uri();
    let method = request.method();
    format!("404 {} {}", method, input_uri)
}
