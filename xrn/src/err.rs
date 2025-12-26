pub type LibxrnResult<T> = Result<T, Box<LibxrnError>>;

#[derive(Debug, PartialEq, Eq, strum::AsRefStr)]
pub enum XrnErrorKind {
    InvalidPrefix,
    InvalidUpper,
    InvalidLower,
    EmptyUpper,
    EmptyAfterUpper,
    //
    AddrInvalidType,
    AddrEmptyAfterType,
    AddrMissingPreIdSep,
    AddrInvalidPreIdSep,
    AddrIdNotANumber,
    AddrMissingPreValueSep,
    //
    PathInvalidType,
    PathMissingTreePrefix,
    PathTreeIdNotANumber,
}

xana_commons_rs::crash_builder!(
    LibxrnError,
    XrnErrorKind,
    xrn_error,
    (extern ParseInt, std::num::ParseIntError),
);

#[cfg(test)]
pub mod test {
    use crate::defs::address::XrnAddr;
    use crate::defs::path_xrn::PathXrn;
    use crate::err::{LibxrnError, LibxrnResult, XrnErrorKind};
    use xana_commons_rs::XanaError;

    pub fn assert_err_kind(res: LibxrnResult<XrnAddr>, expected_kind: XrnErrorKind) {
        match res {
            Ok(res) => panic!("Expected {expected_kind}, got {res}"),
            Err(e) if e.xana_err().kind == expected_kind => {
                // success, we failed!
            }
            Err(e) => panic!("Expected err {expected_kind}, got err {e}"),
        }
    }
}
