use aelita_xrn::err::LibxrnError;
use std::backtrace::Backtrace;
use std::num::TryFromIntError;
use thiserror::Error;

pub type StorDieselResult<T> = Result<T, StorDieselError>;

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

    #[error("StorDieselError_ResultLen expected {0} actual {1}")]
    ResultLen(usize, usize, Backtrace),

    #[error("StorDieselError_TryFromNumber {0}")]
    TryFromNumber(#[from] TryFromIntError, Backtrace),

    #[error("WebError_Diesel {0:?}")]
    Diesel(#[from] diesel::result::Error, Backtrace),
}
