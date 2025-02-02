use aelita_commons::err_utils::{IOEC, IOECSerde, IOECStd};
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

    #[error("StorDieselError_Serde {0} {1}")]
    Serde(PathBuf, serde_json::Error, Backtrace),

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

impl StorDieselError {
    pub fn ioec(path: PathBuf) -> IOEC<Self> {
        IOEC::new(path)
    }
}

impl From<IOECStd> for StorDieselError {
    fn from(IOECStd { path, err }: IOECStd) -> Self {
        Self::IO(path, err)
    }
}

impl From<IOECSerde> for StorDieselError {
    fn from(IOECSerde { path, err }: IOECSerde) -> Self {
        Self::Serde(path, err, Backtrace::capture())
    }
}
