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
    use crate::defs::path_xrn::PathXrn;
    use crate::err::{LibxrnError, LibxrnErrorMeta, LibxrnResult, XrnErrorKind};
    use std::fmt::Display;

    pub fn assert_err_kind(res: LibxrnResult<PathXrn>, expected_kind: XrnErrorKind) {
        match res {
            Ok(res) => panic!("Expected {expected_kind}, got {res}"),
            Err(e) if matches!(std::ops::Deref::deref(&e), LibxrnError { meta: LibxrnErrorMeta::Kind(kind), .. } if *kind == expected_kind) =>
            {
                // success, we failed!
            }
            Err(e) => panic!("Expected err {expected_kind}, got err {e}"),
        }
    }
}
