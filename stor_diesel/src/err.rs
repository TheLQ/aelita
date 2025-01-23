use aelita_xrn::err::LibxrnError;
use std::backtrace::Backtrace;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorDieselError {
    #[error("StorDieselError_LibxrnError {0}")]
    LibxrnError(
        #[from]
        #[backtrace]
        LibxrnError,
    ),
    #[error("StorDieselError_ChronoParse {0}")]
    ChronoParse(#[from] chrono::ParseError, Backtrace),
    #[error("StorDieselError_ValueLen expected <{0} actual {size} value {1}", size = .1.len())]
    ValueLen(usize, String, Backtrace),
}
