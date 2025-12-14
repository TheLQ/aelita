use aelita_stor_diesel::StorDieselError;
use std::backtrace::Backtrace;
use xana_commons_rs::qbittorrent_re::QbitError;
use xana_commons_rs::scraper_re::{PIoReqwestError, PReqwestError};
use xana_commons_rs::{MyBacktrace, SimpleIoError};

pub type StorImportResult<T> = Result<T, StorImportError>;

#[derive(Debug, thiserror::Error)]
pub enum StorImportError {
    #[error(transparent)]
    Io(#[from] SimpleIoError),
    #[error(transparent)]
    Reqwest(#[from] PReqwestError),
    #[error("QbMessage: {msg}")]
    QbMessage { msg: String, bt: Backtrace },
    #[error(transparent)]
    StorDiesel(#[from] StorDieselError),
}

impl From<PIoReqwestError> for StorImportError {
    fn from(value: PIoReqwestError) -> Self {
        match value {
            PIoReqwestError::Io(e) => Self::Io(e),
            PIoReqwestError::Reqwest(e) => Self::Reqwest(e),
        }
    }
}

impl From<QbitError> for StorImportError {
    fn from(value: QbitError) -> Self {
        match value {
            QbitError::Io(e) => Self::Io(e),
            QbitError::Reqwest(e) => Self::Reqwest(e),
            QbitError::Message(msg, bt) => Self::QbMessage { msg, bt },
        }
    }
}

impl MyBacktrace for StorImportError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            Self::Io(e) => e.my_backtrace(),
            Self::Reqwest(e) => e.my_backtrace(),
            Self::StorDiesel(e) => e.my_backtrace(),
            Self::QbMessage { bt, .. } => bt,
        }
    }
}
