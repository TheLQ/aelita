use crate::server::util::{BasicResponse, pretty_basic_page};
use aelita_stor_diesel::StorDieselError;
use aelita_stor_diesel::err::StorDieselErrorKind;
use aelita_xrn::err::{LibxrnError, XrnErrorKind};
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use handlebars::html_escape;
use xana_commons_rs::qbittorrent_re::serde_json;
use xana_commons_rs::tracing_re::error;
use xana_commons_rs::{SimpleIoError, crash_builder, pretty_format_error};

pub type WebResult<R> = Result<R, Box<WebError>>;

#[derive(Debug, strum::AsRefStr, strum::Display)]
pub enum WebErrorKind {
    Crash,
    InvalidUri,
    InvalidXrnTypeForRoute,
    //
    PathXrnMissingPath,
    //
    UnsupportedXrnRoute,
}

crash_builder!(
    WebError,
    WebErrorMeta,
    WebErrorKind,
    // copied from stor_diesel/src/err.rs
    (extern Serde, serde_json::Error),
    (extern Chrono, chrono::ParseError),
    (extern Diesel, aelita_stor_diesel::err_re::DieselError),
    (extern DieselConnect, aelita_stor_diesel::err_re::ConnectionError),
    (extern Postcard, aelita_stor_diesel::err_re::PostcardError),
    (extern SimpleIo, SimpleIoError),
    (extern StdUtf8, std::str::Utf8Error),
    (extern Strum, strum::ParseError),
    (extern TryFromNumber, std::num::TryFromIntError),
    // www unique
    (extern ParseInt, std::num::ParseIntError),
    (extern AxumReject, axum::extract::rejection::PathRejection),
    (extern Deadpool, deadpool_diesel::PoolError),
    (extern DeadpoolInteract, deadpool_diesel::InteractError),
    (extern Handlebars, handlebars::RenderError),
    (extern HandlebarsTemplate, handlebars::TemplateError),
    (mod StorDieselError, StorDieselErrorKind),
    (mod LibxrnError, XrnErrorKind),
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
