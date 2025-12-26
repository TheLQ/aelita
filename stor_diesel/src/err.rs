use xana_commons_rs::{CrashErrKind, crash_builder};

pub type StorDieselResult<T> = Result<T, Box<StorDieselError>>;

#[derive(Debug, PartialEq, strum::AsRefStr)]
pub enum StorDieselErrorKind {
    EmptyPath,
    PathNotAbsolute,
    PathWeird,
    //
    PathXrnRequiresId,
    NotPathXrn,
    UnknownType,
    //
    DatabaseConnectionFailed,
    ExecuteError,
    ResultLen,
    LoadInfileFailed,
    UnexpectedJournalIdForDatabase,
    UnknownComponent,
    UnknownTimestamp,
    UnknownRowCount,
    UnknownVariant,
    ZeroUncommittedJournals,
}

crash_builder!(
    StorDieselError,
    StorDieselErrorKind,
    diesel_err,
    (extern Serde, serde_json::Error),
    (extern Chrono, chrono::ParseError),
    (extern Diesel, diesel::result::Error),
    (extern DieselConnect, diesel::result::ConnectionError),
    (extern Postcard, postcard::Error),
    (extern SimpleIo, xana_commons_rs::SimpleIoError),
    (extern StdUtf8, std::str::Utf8Error),
    (extern Strum, strum::ParseError),
    (extern TryFromNumber, std::num::TryFromIntError),
);

/// Because ? is heavily used. They will probably have the same kind anyway
impl From<diesel::result::Error> for Box<StorDieselError> {
    fn from(value: diesel::result::Error) -> Self {
        StorDieselErrorKind::ExecuteError.build_err(value)
    }
}
