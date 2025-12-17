use crate::server::start_server::CSS_HTML;
use aelita_stor_diesel::StorDieselError;
use aelita_xrn::err::LibxrnError;
use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use handlebars::html_escape;
use std::backtrace::Backtrace;
use std::num::ParseIntError;
use thiserror::Error;
use xana_commons_rs::tracing_re::error;
use xana_commons_rs::{MyBacktrace, pretty_format_error};

pub type WebResult<R> = Result<R, WebError>;

#[derive(Error, Debug)]
#[allow(non_camel_case_types)]
pub enum WebError {
    #[error("WebError_Handlebars {0:?}")]
    Handlebars(#[from] handlebars::RenderError, Backtrace),

    #[error("WebError_DeadpoolInteract {0:?}")]
    DeadpoolInteract(#[from] deadpool_diesel::InteractError, Backtrace),

    #[error("WebError_Deadpool {0:?}")]
    Deadpool(#[from] deadpool_diesel::PoolError, Backtrace),

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
        let body = format!(
            "{CSS_HTML}<div class='section'><h1 class='title'>500</h1><pre>{}</pre></div>",
            html_escape(&pretty)
        );
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(body))
            .unwrap()
    }
}

impl MyBacktrace for WebError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            WebError::Handlebars(_, bt) => bt,
            WebError::DeadpoolInteract(_, bt) => bt,
            WebError::Deadpool(_, bt) => bt,
            WebError::Strum(_, bt) => bt,
            WebError::StorDiesel(e) => e.my_backtrace(),
            WebError::Libxrn(e) => e.my_backtrace(),
            WebError::ParseInt(_, bt) => bt,
            WebError::Assert(_, bt) => bt,
            WebError::UnsupportedXrnRoute(_, bt) => bt,
        }
    }
}
