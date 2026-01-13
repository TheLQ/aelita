use aelita_stor_diesel::StorDieselError;
use aelita_stor_diesel::err::StorDieselErrorKind;
use xana_commons_rs::CrashErr;
use xana_commons_rs::qbittorrent_re::QbitError;
use xana_commons_rs::scraper_re::{PIoReqwestError, PReqwestError};
use xana_commons_rs::{CrashErrKind, crash_builder};
use xana_fs_indexer_rs::IndexerErrorKind;

pub type StorImportResult<T> = Result<T, Box<StorImportError>>;

#[derive(Debug, strum::AsRefStr)]
pub enum StorImportErrorKind {
    MalformedQbMetadata,
    BadQbGetTorrents,
    QbInit,
    QbAuth,
    InvalidCompressedPaths,
    DieselFailed,
    InvalidQbMetadata,
    InvalidQbTorrents,
    InvalidChangeOp,
    //
    MigrationMissingCreate,
    MigrationMissingEnd,
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
    (extern SimpleIo, xana_commons_rs::SimpleIoError),
    (extern Reqwest, xana_commons_rs::error_re::ReqwestError),
    (extern StdUtf8, std::str::Utf8Error),
    (extern Strum, strum::ParseError),
    (extern TryFromNumber, std::num::TryFromIntError),
    // import unique
    (mod StorDieselError, StorDieselErrorKind),
    (mod IndexerError, IndexerErrorKind),
);

impl From<Box<StorDieselError>> for Box<StorImportError> {
    fn from(value: Box<StorDieselError>) -> Self {
        StorImportErrorKind::DieselFailed.xana(value)
    }
}

impl StorImportErrorKind {
    pub fn reqwest_pio(self, e: PIoReqwestError) -> Box<StorImportError> {
        match e {
            PIoReqwestError::Reqwest(e) => self.reqwest(e),
            PIoReqwestError::Io(e) => self.build_err(e),
        }
    }

    pub fn reqwest(self, PReqwestError { error, bt }: PReqwestError) -> Box<StorImportError> {
        <Self as CrashErrKind>::CrashError::new(self, Some(error.into()), None, "", bt).into()
    }

    pub fn qbit(self, e: QbitError) -> Box<StorImportError> {
        match e {
            QbitError::Reqwest(e) => self.reqwest(e),
            QbitError::Io(e) => self.build_err(e),
            QbitError::Message(message, bt) => {
                <Self as CrashErrKind>::CrashError::new(self, None, None, message, bt).into()
            }
        }
    }
}
