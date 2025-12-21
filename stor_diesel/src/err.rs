use std::backtrace::Backtrace;
use std::num::TryFromIntError;
use xana_commons_rs::{MyBacktrace, SimpleIoError, crash_builder};

pub type StorDieselResult<T> = Result<T, Box<StorDieselError>>;

#[derive(Debug, strum::AsRefStr)]
pub enum StorDieselErrorKind {
    Crash,
    //
    EmptyPath,
    PathNotAbsolute,
    PathWeird,
    //
    PathXrnRequiresId,
    NotPathXrn,
    UnknownType,
    //
    ResultLen,
    UnexpectedJournalIdForDatabase,
    UnknownComponent,
    UnknownTimestamp,
    ZeroUncommittedJournals,
}

crash_builder!(
    StorDieselError,
    StorDieselErrorMeta,
    StorDieselErrorKind,
    (Serde, serde_json::Error),
    (Chrono, chrono::ParseError),
    (Diesel, diesel::result::Error),
    (DieselConnect, diesel::result::ConnectionError),
    (TryFromNumber, TryFromIntError),
    (Postcard, postcard::Error),
    (SimpleIo, SimpleIoError),
    (StdUtf8, std::str::Utf8Error),
    (Strum, strum::ParseError),
);

impl MyBacktrace for StorDieselError {
    fn my_backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}
