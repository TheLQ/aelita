use aelita_stor_diesel::models::StorDate;
use axum::http::header::ACCEPT;
use axum::http::{HeaderMap, HeaderValue};
use chrono::{DateTime, FixedOffset, Local, SecondsFormat};
use std::borrow::Borrow;

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
