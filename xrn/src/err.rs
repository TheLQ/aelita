use crate::defs::address::XrnAddr;
use std::backtrace::Backtrace;
use thiserror::Error;
use xana_commons_rs::MyBacktrace;

pub type LibxrnResult<T> = Result<T, LibxrnError>;

#[derive(Error, Debug)]
pub enum LibxrnError {
    #[error("libxrn_ParseShort {0}")]
    ParseShort(String, Backtrace),
    #[error("libxrn_ParsePrefix {0}")]
    ParsePrefix(String, Backtrace),
    #[error("libxrn_MissingSeparator {0}")]
    MissingSeparator(String, Backtrace),
    #[error("libxrn_InvalidType {0}")]
    InvalidType(String, Backtrace),
    #[error("libxrn_AddrInvalidType {0}")]
    AddrInvalidType(XrnAddr, Backtrace),
    #[error("libxrn_AddrMissingSeparator {0}")]
    AddrMissingSeparator(XrnAddr, Backtrace),
    #[error("libxrn_AddrNotANumber {0}")]
    AddrNotANumber(XrnAddr, Backtrace),
}

impl MyBacktrace for LibxrnError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            LibxrnError::ParseShort(_, bt) => bt,
            LibxrnError::ParsePrefix(_, bt) => bt,
            LibxrnError::MissingSeparator(_, bt) => bt,
            LibxrnError::InvalidType(_, bt) => bt,
            LibxrnError::AddrInvalidType(_, bt) => bt,
            LibxrnError::AddrMissingSeparator(_, bt) => bt,
            LibxrnError::AddrNotANumber(_, bt) => bt,
        }
    }
}
