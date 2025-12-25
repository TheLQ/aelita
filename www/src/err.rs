use crate::server::util::{BasicResponse, pretty_basic_page};
use aelita_stor_diesel::StorDieselError;
use aelita_stor_diesel::err::StorDieselErrorKind;
use aelita_xrn::err::{LibxrnError, XrnErrorKind};
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use handlebars::html_escape;
use xana_commons_rs::tracing_re::error;
use xana_commons_rs::{SimpleIoError, crash_builder, pretty_format_error};

pub type WebResult<R> = Result<R, Box<WebError>>;

#[derive(Debug, strum::AsRefStr)]
pub enum WebErrorKind {
    InvalidUri,
    InvalidXrnTypeForRoute,
    XrnParseFailed,
    //
    PathXrnMissingPath,
    //
    UnsupportedXrnRoute,
    //
    HandlebarsInitFailed,
    HandlebarsRenderFailed,
    //
    StorError,
    SerdeToJsonResponse,
}

crash_builder!(
    WebError,
    WebErrorKind,
    web_error,
    // copied from stor_diesel/src/err.rs
    (extern Serde, xana_commons_rs::qbittorrent_re::serde_json::Error),
    (extern Chrono, chrono::ParseError),
    (extern Diesel, aelita_stor_diesel::err_re::DieselError),
    (extern DieselConnect, aelita_stor_diesel::err_re::ConnectionError),
    (extern Postcard, aelita_stor_diesel::err_re::PostcardError),
    (extern SimpleIo, xana_commons_rs::SimpleIoError),
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
pub use web_error::Cause as WebErrorCause;

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
