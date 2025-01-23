use aelita_commons::err_utils::pretty_error;
use aelita_xrn::err::LibxrnError;
use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use handlebars::html_escape;
use std::backtrace::Backtrace;
use thiserror::Error;

pub type WebResult<R> = Result<R, WebError>;

#[derive(Error, Debug)]
#[allow(non_camel_case_types)]
pub enum WebError {
    #[error("WebError_Handlebars {0:#?}")]
    Handlebars(#[from] handlebars::RenderError, Backtrace),

    #[error("WebError_DeadpoolInteract {0:#?}")]
    DeadpoolInteract(#[from] deadpool_diesel::InteractError, Backtrace),

    #[error("WebError_Deadpool {0:#?}")]
    Deadpool(#[from] deadpool_diesel::PoolError, Backtrace),

    #[error("WebError_Diesel {0:#?}")]
    Diesel(
        #[from] aelita_stor_diesel::diesel_re::result::Error,
        Backtrace,
    ),

    // #[error("Libxrn {}", pretty_error(.0))]
    #[error("WebError_Libxrn {0:#?}")]
    Libxrn(#[from] LibxrnError),

    #[error("XrnRegistry_IsEmpty {0:#?}")]
    XrnRegistry_IsEmpty(Backtrace),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let body = format!(
            "<h1>500</h1><pre>{}</pre>",
            html_escape(&pretty_error(self))
        );
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(body))
            .unwrap()
    }
}
