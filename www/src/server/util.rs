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

pub fn pretty_basic_page(title: impl Display, body: impl Display) -> String {
    // {CSS_HTML}
    // <meta name="viewport" content="width=device-width, initial-scale=1">
    // <section class='section'>
    format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8">

            <title>{title}</title>
            <link rel='stylesheet' href='https://cdn.jsdelivr.net/npm/bulma@1.0.4/css/bulma.min.css'>
            <link rel="stylesheet" href="/scripts/searcher.css">
        </head>
        <body>

            <h1 class='title'>{title}</h1>
            {body}

        </body>
        </html>"#
    )
}

pub struct BasicResponse(pub StatusCode, pub mime::Mime, pub Body);

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
