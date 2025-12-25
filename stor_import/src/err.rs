use aelita_stor_diesel::StorDieselError;
use std::backtrace::Backtrace;
use xana_commons_rs::qbittorrent_re::QbitError;
use xana_commons_rs::scraper_re::{PIoReqwestError, PReqwestError};
use xana_commons_rs::{MyBacktrace, SimpleIoError, crash_builder};

pub type StorImportResult<T> = Result<T, Box<StorImportError>>;

#[derive(Debug, strum::AsRefStr)]
pub enum StorImportErrorKind {
    MalformedQbMetadata,
}
crash_builder!(
    StorImportError,
    StorImportErrorKind,
    stor_import,
    // copied from stor_diesel/src/err.rs
    (extern Serde, serde_json::Error),
    (extern Chrono,  aelita_stor_diesel::err_re::ChronoError),
    (extern Diesel, aelita_stor_diesel::err_re::DieselError),
    (extern DieselConnect, aelita_stor_diesel::err_re::ConnectionError),
    (extern Postcard, aelita_stor_diesel::err_re::PostcardError),
    (extern SimpleIo, SimpleIoError),
    (extern StdUtf8, std::str::Utf8Error),
    (extern Strum, strum::ParseError),
    (extern TryFromNumber, std::num::TryFromIntError),
    // import unique
);
