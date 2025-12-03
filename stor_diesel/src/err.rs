use std::backtrace::Backtrace;
use std::num::TryFromIntError;
use thiserror::Error;
use xana_commons_rs::{MyBacktrace, SimpleIoError};

pub type StorDieselResult<T> = Result<T, StorDieselError>;

#[derive(Error, Debug)]
pub enum StorDieselError {
    #[error("StorDieselError_IO {0}")]
    IO(#[from] SimpleIoError),

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
    pub fn query_fail(input: impl Into<String>) -> Self {
        Self::QueryFail(input.into(), Backtrace::capture())
    }
}

impl MyBacktrace for StorDieselError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            StorDieselError::IO(e) => e.my_backtrace(),
            StorDieselError::ChronoParse(_, bt) => bt,
            StorDieselError::ResultLen { backtrace, .. } => backtrace,
            StorDieselError::TryFromNumber(_, bt) => bt,
            StorDieselError::Diesel(_, bt) => bt,
            StorDieselError::QueryFail(_, bt) => bt,
        }
    }
}
