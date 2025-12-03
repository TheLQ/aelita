use aelita_commons::err_utils::{IOEC, IOECSerde, IOECStd, xbt};
use aelita_xrn::err::LibxrnError;
use std::backtrace::Backtrace;
use std::io;
use std::num::TryFromIntError;
use std::path::PathBuf;
use thiserror::Error;

pub type StorDieselResult<T> = Result<T, StorDieselError>;

#[derive(Error, Debug)]
pub enum StorDieselError {
    #[error("StorDieselError_IO {0} {1}")]
    IO(PathBuf, io::Error),

    #[error("StorDieselError_LibxrnError {0}")]
    LibxrnError(
        #[from]
        #[backtrace]
        LibxrnError,
    ),

    #[error("StorDieselError_ChronoParse {0}")]
    ChronoParse(#[from] chrono::ParseError, Backtrace),

    #[error("StorDieselError_ResultLen actual {actual} expected {expected}")]
    ResultLen {
        actual: usize,
        expected: usize,
        backtrace: Backtrace,
    },

    #[error("StorDieselError_TryFromNumber {0}")]
    TryFromNumber(#[from] TryFromIntError, Backtrace),

    #[error("WebError_Diesel {0:?}")]
    Diesel(#[from] diesel::result::Error, Backtrace),

    #[error("QueryFail {0:?}")]
    QueryFail(String, Backtrace),
}

impl StorDieselError {
    pub fn ioec(path: impl Into<PathBuf>) -> IOEC<Self> {
        IOEC::new(path.into())
    }

    pub fn query_fail(input: impl Into<String>) -> Self {
        Self::QueryFail(input.into(), Backtrace::capture())
    }
}

impl From<IOECStd> for StorDieselError {
    fn from(IOECStd { path, err }: IOECStd) -> Self {
        Self::IO(path, err)
    }
}
