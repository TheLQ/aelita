use crate::defs::address::XrnAddr;
use std::backtrace::Backtrace;
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::Utf8Error;
use strum::{AsRefStr, AsStaticStr};
use thiserror::Error;
use xana_commons_rs::MyBacktrace;

pub type LibxrnResult<T> = Result<T, LibxrnError>;

#[derive(Error, Debug)]
pub struct LibxrnError {
    pub kind: XrnErrorKind,
    name: String,
    // description: Option<String>,
    backtrace: Backtrace,
}

#[derive(Debug, PartialEq, Eq, AsRefStr, strum::Display)]
pub enum XrnErrorKind {
    AddrPrefix,
    AddrInvalidType,
    AddrEmptyValue,
    //
    PathInvalidInputType,
    PathInvalidType,
    PathEmptyValue,
    PathTreeIdNotANumber,
    //
    SpaceInvalidInputType,
    SpaceInvalidType,
    //
    // InvalidUtf8,
    // InvalidInt,
    InvalidTreeId,
}

impl LibxrnError {
    // pub fn utf8_parse_addr(addr: impl Borrow<XrnAddr>) -> impl FnOnce(Utf8Error) -> Self {
    //     let name = addr.borrow().to_string();
    //     move |_| Self {
    //         kind: XrnErrorKind::InvalidUtf8,
    //         name,
    //         backtrace: Backtrace::capture(),
    //     }
    // }
    //
    // pub fn parse_int_addr(addr: impl Borrow<XrnAddr>) -> impl FnOnce(ParseIntError) -> Self {
    //     let name = addr.borrow().to_string();
    //     move |_| Self {
    //         kind: XrnErrorKind::InvalidInt,
    //         name,
    //         backtrace: Backtrace::capture(),
    //     }
    // }
}

impl MyBacktrace for LibxrnError {
    fn my_backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl Display for LibxrnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LibxrnError_{} for {}", self.kind.as_ref(), self.name)
    }
}

impl XrnErrorKind {
    pub fn err_addr(self, addr: XrnAddr) -> LibxrnError {
        LibxrnError {
            kind: self,
            name: addr.to_string(),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn err_raw(self, raw: impl Into<String>) -> LibxrnError {
        LibxrnError {
            kind: self,
            name: raw.into(),
            backtrace: Backtrace::capture(),
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::err::{LibxrnError, XrnErrorKind};
    use std::fmt::Display;

    pub fn assert_err_kind<T>(res: Result<T, LibxrnError>, expected_kind: XrnErrorKind)
    where
        T: Display,
    {
        match res {
            Ok(res) => panic!("Expected {expected_kind}, got {res}"),
            Err(LibxrnError { kind, .. }) if kind == expected_kind => {
                // success, we failed!
            }
            Err(e) => panic!("Expected err {expected_kind}, got err {e}"),
        }
    }
}
