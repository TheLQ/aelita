use strum::AsRefStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibxrnError {
    #[error("libxrn_ParseShort {0}")]
    ParseShort(String),
    #[error("libxrn_ParsePrefix {0}")]
    ParsePrefix(String),
    #[error("libxrn_ParsePrefixAfter {0}")]
    ParsePrefixAfter(String),
    #[error("libxrn_InvalidType {0}")]
    InvalidType(String),
}

impl LibxrnError {}
