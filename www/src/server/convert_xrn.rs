use crate::err::WebError;
use aelita_xrn::defs::address::XrnAddr;
use aelita_xrn::defs::common::SubXrn;
use axum::extract::{FromRequestParts, Path};
use axum::http::StatusCode;
use axum::http::request::Parts;
use serde::de::StdError;
use std::fmt::Debug;
use std::str::FromStr;
use xana_commons_rs::pretty_format_error;
use xana_commons_rs::tracing_re::{error, info, trace, warn};

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
    T: TryFrom<XrnAddr> + SubXrn + Debug,
    <T as TryFrom<XrnAddr>>::Error: std::fmt::Debug + StdError,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Path(xrn_value) = Path::<String>::from_request_parts(parts, _state)
            .await
            .map_err(|e| WebError::assert(format!("failed 1st xrn {e}")))?;
        let best_addr = XrnAddr::new(T::atype(), xrn_value);

        // xrn type assumed as axum only gives us the xrn-value
        // validate type is correct
        let full_uri = parts.uri.to_string();
        let (prefix_sep, addr_raw) = full_uri.split_at(1);
        if prefix_sep != "/" {
            return Err(WebError::assert("uri"));
        }
        let raw_addr = XrnAddr::from_str(&addr_raw)?;
        if best_addr.atype() != raw_addr.atype() {
            return Err(WebError::assert(format!(
                "unexpected type {} vs {}",
                best_addr.atype(),
                raw_addr.atype()
            )));
        }

        trace!("building addr with {}", best_addr);
        let result =
            T::try_from(best_addr).map_err(|e| WebError::assert(format!("parse fail {e}")))?;
        Ok(XrnFromUrl(result))
    }
}
