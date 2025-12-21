use crate::server::util::{BasicResponse, pretty_basic_page};
use aelita_stor_diesel::StorDieselError;
use aelita_stor_diesel::err::{StorDieselErrorKind, StorDieselErrorMeta};
use aelita_xrn::err::{LibxrnError, LibxrnErrorMeta, XrnErrorKind};
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use handlebars::html_escape;
use std::backtrace::Backtrace;
use thiserror::Error;
use xana_commons_rs::qbittorrent_re::serde_json;
use xana_commons_rs::tracing_re::error;
use xana_commons_rs::{
    MyBacktrace, SimpleIoError, crash_builder, crash_into_crash, pretty_format_error,
};

pub type WebResult<R> = Result<R, Box<WebError>>;

#[derive(Debug, strum::AsRefStr)]
pub enum WebErrorKind {
    Crash,
    InvalidUri,
    InvalidXrnTypeForRoute,
}

crash_builder!(
    WebError,
    WebErrorMeta,
    WebErrorKind,
    // copied from stor_diesel/src/err.rs
    (Serde, serde_json::Error),
    (Chrono, chrono::ParseError),
    (Diesel, aelita_stor_diesel::err_re::DieselError),
    (DieselConnect, aelita_stor_diesel::err_re::ConnectionError),
    (Postcard, aelita_stor_diesel::err_re::PostcardError),
    (SimpleIo, SimpleIoError),
    (StdUtf8, std::str::Utf8Error),
    (Strum, strum::ParseError),
    (TryFromNumber, std::num::TryFromIntError),
    // www unique
    (ParseInt, std::num::ParseIntError),
    (AxumReject, axum::extract::rejection::PathRejection),
    (Deadpool, deadpool_diesel::PoolError),
    (DeadpoolInteract, deadpool_diesel::InteractError),
    (Handlebars, handlebars::RenderError),
    (HandlebarsTemplate, handlebars::TemplateError),
);
crash_into_crash!(
    LibxrnError,
    LibxrnErrorMeta,
    XrnErrorKind,
    WebError,
    WebErrorMeta,
    WebErrorKind,
    []
);
crash_into_crash!(
    StorDieselError,
    StorDieselErrorMeta,
    StorDieselErrorKind,
    WebError,
    WebErrorMeta,
    WebErrorKind,
    []
);

// #[derive(Error, Debug)]
// #[allow(non_camel_case_types)]
// pub enum WebError {
//     #[error("WebError_Axum {0:?}")]
//     Axum(#[from] axum::http::Error, Backtrace),
//
//     #[error("WebError_Handlebars {0:?}")]
//     Handlebars(#[from] handlebars::RenderError, Backtrace),
//
//     #[error("WebError_HandlebarsTemplate {0:?}")]
//     HandlebarsTemplate(#[from] handlebars::TemplateError, Backtrace),
//
//     #[error("WebError_DeadpoolInteract {0:?}")]
//     DeadpoolInteract(#[from] deadpool_diesel::InteractError, Backtrace),
//
//     #[error("WebError_Deadpool {0:?}")]
//     Deadpool(#[from] deadpool_diesel::PoolError, Backtrace),
//
//     #[error("WebError_SerdeJson {0:?}")]
//     SerdeJson(#[from] serde_json::Error, Backtrace),
//
//     #[error("WebError_Axum {0:?}")]
//     SimpleIo(#[from] SimpleIoError),
//
//     #[error("WebError_Strum {0:?}")]
//     Strum(#[from] strum::ParseError, Backtrace),
//
//     #[error("WebError_StorDiesel {0}")]
//     StorDiesel(
//         #[from]
//         #[backtrace]
//         StorDieselError,
//     ),
//
//     // #[error("Libxrn {}", pretty_error(.0))]
//     #[error("WebError_Libxrn {0:?}")]
//     Libxrn(
//         #[from]
//         #[backtrace]
//         LibxrnError,
//     ),
//
//     #[error("WebError_ParseInt {0:?}")]
//     ParseInt(#[from] ParseIntError, Backtrace),
//
//     #[error("Assert {0}")]
//     Assert(String, Backtrace),
//
//     #[error("UnsupportedXrnRoute {0}")]
//     UnsupportedXrnRoute(String, Backtrace),
// }

impl IntoResponse for Box<WebError> {
    fn into_response(self) -> Response {
        let pretty = pretty_format_error(&*self);
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
