use aelita_commons::tracing_re::warn;
use aelita_xrn::defs::address::XrnAddr;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

pub struct XrnFromUrl(pub XrnAddr);

impl XrnFromUrl {
    pub fn into_inner(self) -> XrnAddr {
        warn!("decoded into {:?}", self.0);
        self.0
    }
}

impl<S> FromRequestParts<S> for XrnFromUrl
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let full_uri = parts.uri.path();
        // warn!("parts {}", pars);
        let (comma, remain) = full_uri.split_at(1);
        if comma != "/" {
            return Err(fail_response("uri"));
        }

        let addr_raw = if let Some(remain_start) = remain.find("/") {
            let (addr_raw, _) = remain.split_at(remain_start);
            addr_raw
        } else {
            remain
        };

        match XrnAddr::from_str(addr_raw) {
            Ok(addr) => Ok(XrnFromUrl(addr)),
            Err(e) => Err(fail_response(format!("invalid xrn {:?}", e))),
        }
    }
}

fn fail_response(message: impl Into<String>) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, message.into())
}
