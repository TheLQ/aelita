use crate::err::{WebError, WebErrorKind};
use aelita_xrn::defs::common::SubXrnImpl;
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use xana_commons_rs::tracing_re::trace;
use xana_commons_rs::{CrashErrKind, ResultXanaMap};

/// Axum extractor to parse xrn directly from the path
pub struct XrnFromUrl<Xrn>(pub Xrn)
where
    Xrn: SubXrnImpl;

impl<Xrn> XrnFromUrl<Xrn>
where
    Xrn: SubXrnImpl,
{
    pub fn into_inner(self) -> Xrn {
        self.0
    }
}

impl<S, Xrn> FromRequestParts<S> for XrnFromUrl<Xrn>
where
    S: Send + Sync,
    Xrn: SubXrnImpl,
{
    type Rejection = Box<WebError>;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // reassemble Xrn after Axum router took "xrn:key"
        let Path(mut xrn_value) = Path::<String>::from_request_parts(parts, _state)
            .await
            .xana_err(WebErrorKind::PathXrnMissingPath)?;
        xrn_value.insert_str(0, &format!("xrn:{}", Xrn::UPPER.as_ref()));

        trace!("building addr with {xrn_value}");
        let result = Xrn::from_str(&xrn_value).map_err(WebErrorKind::XrnParseFailed.xana_map())?;
        Ok(XrnFromUrl(result))
    }
}
