use std::num::TryFromIntError;
use xana_commons_rs::{SimpleIoError, crash_builder};

pub type StorDieselResult<T> = Result<T, Box<StorDieselError>>;

#[derive(Debug, strum::AsRefStr, strum::Display)]
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
    (extern Serde, serde_json::Error),
    (extern Chrono, chrono::ParseError),
    (extern Diesel, diesel::result::Error),
    (extern DieselConnect, diesel::result::ConnectionError),
    (extern Postcard, postcard::Error),
    (extern SimpleIo, SimpleIoError),
    (extern StdUtf8, std::str::Utf8Error),
    (extern Strum, strum::ParseError),
    (extern TryFromNumber, TryFromIntError),
);
