use crate::server::util::{BasicResponse, pretty_basic_page};
use aelita_stor_diesel::StorDieselError;
use aelita_xrn::err::LibxrnError;
use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use handlebars::html_escape;
use std::backtrace::Backtrace;
use std::num::ParseIntError;
use thiserror::Error;
use xana_commons_rs::qbittorrent_re::serde_json;
use xana_commons_rs::tracing_re::error;
use xana_commons_rs::{MyBacktrace, pretty_format_error};

pub type WebResult<R> = Result<R, WebError>;

#[derive(Error, Debug)]
#[allow(non_camel_case_types)]
pub enum WebError {
    #[error("WebError_Axum {0:?}")]
    Axum(#[from] axum::http::Error, Backtrace),

    #[error("WebError_Handlebars {0:?}")]
    Handlebars(#[from] handlebars::RenderError, Backtrace),

    #[error("WebError_DeadpoolInteract {0:?}")]
    DeadpoolInteract(#[from] deadpool_diesel::InteractError, Backtrace),

    #[error("WebError_Deadpool {0:?}")]
    Deadpool(#[from] deadpool_diesel::PoolError, Backtrace),

    #[error("WebError_SerdeJson {0:?}")]
    SerdeJson(#[from] serde_json::Error, Backtrace),

    #[error("WebError_Strum {0:?}")]
    Strum(#[from] strum::ParseError, Backtrace),

    #[error("WebError_StorDiesel {0}")]
    StorDiesel(
        #[from]
        #[backtrace]
        StorDieselError,
    ),

    // #[error("Libxrn {}", pretty_error(.0))]
    #[error("WebError_Libxrn {0:?}")]
    Libxrn(
        #[from]
        #[backtrace]
        LibxrnError,
    ),

    #[error("WebError_ParseInt {0:?}")]
    ParseInt(#[from] ParseIntError, Backtrace),

    #[error("Assert {0}")]
    Assert(String, Backtrace),

    #[error("UnsupportedXrnRoute {0}")]
    UnsupportedXrnRoute(String, Backtrace),
}

impl WebError {
    pub fn unsupported_xrn_route(value: impl Into<String>) -> Self {
        Self::UnsupportedXrnRoute(value.into(), Backtrace::capture())
    }

    pub fn assert(value: impl Into<String>) -> Self {
        Self::Assert(value.into(), Backtrace::capture())
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let pretty = pretty_format_error(&self);
        error!("Status 500 {}", pretty);
        let body = pretty_basic_page("500", format!("<pre>{}</pre>", html_escape(&pretty)));

        BasicResponse(
            StatusCode::INTERNAL_SERVER_ERROR,
            mime::HTML,
            Body::from(body),
        )
        .into_response()
    }
}

impl MyBacktrace for WebError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            WebError::Axum(_, bt) => bt,
            WebError::Handlebars(_, bt) => bt,
            WebError::DeadpoolInteract(_, bt) => bt,
            WebError::Deadpool(_, bt) => bt,
            WebError::SerdeJson(_, bt) => bt,
            WebError::Strum(_, bt) => bt,
            WebError::StorDiesel(e) => e.my_backtrace(),
            WebError::Libxrn(e) => e.my_backtrace(),
            WebError::ParseInt(_, bt) => bt,
            WebError::Assert(_, bt) => bt,
            WebError::UnsupportedXrnRoute(_, bt) => bt,
        }
    }
}
