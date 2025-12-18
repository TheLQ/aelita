use serde_json::Error;
use std::backtrace::Backtrace;
use std::num::TryFromIntError;
use thiserror::Error;
use xana_commons_rs::{MyBacktrace, SimpleIoError};

pub type StorDieselResult<T> = Result<T, StorDieselError>;

#[derive(Error, Debug)]
pub enum StorDieselError {
    #[error("StorDieselError_IO {0}")]
    IO(#[from] SimpleIoError),

    #[error("StorDieselError_Serde {e}\nsource\n{}", .message.as_ref().unwrap())]
    Serde {
        e: serde_json::Error,
        backtrace: Backtrace,
        message: Option<String>,
    },

    #[error("StorDieselError_Postcard {0}")]
    Postcard(#[from] postcard::Error, Backtrace),

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

    #[error("Unknown component(s) {0}")]
    UnknownComponent(String, Backtrace),
}

impl StorDieselError {
    pub fn query_fail(input: impl Into<String>) -> Self {
        Self::QueryFail(input.into(), Backtrace::capture())
    }

    pub fn serde_extract(e: serde_json::Error, message: impl Into<String>) -> Self {
        Self::Serde {
            e,
            message: Some(message.into()),
            backtrace: Backtrace::capture(),
        }
    }
}

impl MyBacktrace for StorDieselError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            StorDieselError::IO(e) => e.my_backtrace(),
            StorDieselError::Serde { backtrace, .. } => backtrace,
            StorDieselError::Postcard(_, bt) => bt,
            StorDieselError::ChronoParse(_, bt) => bt,
            StorDieselError::ResultLen { backtrace, .. } => backtrace,
            StorDieselError::TryFromNumber(_, bt) => bt,
            StorDieselError::Diesel(_, bt) => bt,
            StorDieselError::QueryFail(_, bt) => bt,
            StorDieselError::UnknownComponent(_, bt) => bt,
        }
    }
}

impl From<serde_json::Error> for StorDieselError {
    fn from(e: Error) -> Self {
        Self::Serde {
            e,
            message: None,
            backtrace: Backtrace::capture(),
        }
    }
}
