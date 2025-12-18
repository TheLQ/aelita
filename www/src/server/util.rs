use axum::body::Body;
use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use std::borrow::Borrow;
use std::fmt::Display;

pub fn is_accepts_html(parts: impl Borrow<HeaderMap>) -> bool {
    let parts = parts.borrow();
    match parts.get(ACCEPT) {
        Some(value) if is_value_contains_html(value) => true,
        Some(_) | None => false,
    }
}

fn is_value_contains_html(value: &HeaderValue) -> bool {
    let Ok(value) = value.to_str() else {
        return false;
    };
    value.contains("text/html")
}

pub const CSS_HTML: &str =
    "<link rel='stylesheet' href='https://cdn.jsdelivr.net/npm/bulma@1.0.4/css/bulma.min.css'>";

pub fn pretty_basic_page(title: impl Display, body: impl Display) -> String {
    format!("{CSS_HTML}<section class='section'><h1 class='title'>{title}</h1>{body}")
}

pub struct BasicResponse(pub StatusCode, pub mime::Name<'static>, pub Body);

impl IntoResponse for BasicResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .status(self.0)
            .header(CONTENT_TYPE, self.1.to_string())
            .body(self.2)
            // welp we can't do anything with it
            .unwrap()
    }
}
