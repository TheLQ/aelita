use crate::defs::address::XrnAddr;
use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};
use strum::AsRefStr;
use thiserror::Error;
use xana_commons_rs::MyBacktrace;

pub type LibxrnResult<T> = Result<T, Box<LibxrnError>>;

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
    SpaceEmptyValue,
    InvalidSpaceId,
    //
    // InvalidUtf8,
    // InvalidInt,
    InvalidTreeId,
}

xana_commons_rs::crash_builder!(LibxrnError, LibxrnErrorMeta, XrnErrorKind,);

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

#[cfg(test)]
pub mod test {
    use crate::err::{LibxrnError, LibxrnResult, XrnErrorKind};
    use std::fmt::Display;

    #[test]
    fn some() {
        assert_eq!(0, std::mem::size_of::<Result<(), Box<LibxrnError>>>());
    }

    pub fn assert_err_kind<T>(res: Result<T, LibxrnError>, expected_kind: XrnErrorKind)
    where
        T: Display,
    {
        assert_eq!(0, std::mem::size_of::<LibxrnResult<()>>());

        match res {
            Ok(res) => panic!("Expected {expected_kind}, got {res}"),
            Err(LibxrnError { kind, .. }) if kind == expected_kind => {
                // success, we failed!
            }
            Err(e) => panic!("Expected err {expected_kind}, got err {e}"),
        }
    }
}
