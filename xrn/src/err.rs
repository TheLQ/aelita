use crate::defs::address::XrnAddr;
use std::backtrace::Backtrace;
use strum::AsRefStr;
use thiserror::Error;

pub type LibxrnResult<T> = Result<T, LibxrnError>;

#[derive(Error, Debug)]
pub enum LibxrnError {
    #[error("libxrn_ParseShort {0}")]
    ParseShort(String),
    #[error("libxrn_ParsePrefix {0}")]
    ParsePrefix(String),
    #[error("libxrn_MissingSeparator {0}")]
    MissingSeparator(String, Backtrace),
    #[error("libxrn_InvalidType {0}")]
    InvalidType(String),
    #[error("libxrn_AddrInvalidType {0}")]
    AddrInvalidType(XrnAddr, Backtrace),
    #[error("libxrn_AddrMissingSeparator {0}")]
    AddrMissingSeparator(XrnAddr),
    #[error("libxrn_AddrNotANumber {0}")]
    AddrNotANumber(XrnAddr),
}

impl LibxrnError {}
