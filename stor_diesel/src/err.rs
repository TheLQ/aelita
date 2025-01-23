use aelita_xrn::err::LibxrnError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorDieselError {
    #[error("StorDieselError_LibxrnError {0}")]
    LibxrnError(#[from] LibxrnError),
    #[error("StorDieselError_ChronoParse {0}")]
    ChronoParse(#[from] chrono::ParseError),
}
