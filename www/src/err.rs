use thiserror::Error;

pub type WResult<R> = Result<R, WebError>;

#[derive(Error, Debug)]
pub enum WebError {
    #[error("WebError_Handlebars {0:#?}")]
    Handlebars(#[from] handlebars::RenderError),
}
