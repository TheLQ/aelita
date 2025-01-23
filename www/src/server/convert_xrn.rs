use aelita_commons::err_utils::pretty_error;
use aelita_commons::tracing_re::{error, trace, warn};
use aelita_xrn::defs::address::XrnAddr;
use axum::RequestPartsExt;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use serde::de::StdError;
use serde::{Deserialize, Deserializer};
use std::fmt::Debug;
use std::path::Path;
use std::str::FromStr;

/// Axum extractor to parse xrn directly from the path
pub struct XrnFromUrl<T>(pub T)
where
    T: TryFrom<XrnAddr> + Debug;

impl<T> XrnFromUrl<T>
where
    T: TryFrom<XrnAddr> + Debug,
{
    pub fn into_inner(self) -> T {
        warn!("decoded into {:?}", self.0);
        self.0
    }
}

impl<S, T> FromRequestParts<S> for XrnFromUrl<T>
where
    S: Send + Sync,
    T: TryFrom<XrnAddr> + Debug,
    <T as TryFrom<XrnAddr>>::Error: std::fmt::Debug + StdError,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let full_uri = parts.uri.to_string();
        // warn!("uri {:?}", parts);
        // warn!("full {}", full_uri);
        let (prefix_sep, addr_raw) = full_uri.split_at(1);
        if prefix_sep != "/" {
            return Err(fail_response("uri"));
        }

        trace!("building addr with {}", addr_raw);
        let addr = XrnAddr::from_str(addr_raw)
            .map_err(|e| fail_response(format!("invalid xrn {}", pretty_error(e))))?;

        let mepls = T::try_from(addr)
            .map_err(|e| fail_response(format!("parse fail {}", pretty_error(e))))?;
        Ok(XrnFromUrl(mepls))
    }
}

fn fail_response(message: impl Into<String>) -> (StatusCode, String) {
    let message = message.into();
    error!("{}", message);
    (StatusCode::INTERNAL_SERVER_ERROR, message)
}
