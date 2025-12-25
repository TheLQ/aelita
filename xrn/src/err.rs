pub type LibxrnResult<T> = Result<T, Box<LibxrnError>>;

#[derive(Debug, PartialEq, Eq, strum::AsRefStr)]
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

xana_commons_rs::crash_builder!(LibxrnError, XrnErrorKind, xrn_error,);

#[cfg(test)]
pub mod test {
    use crate::defs::path_xrn::PathXrn;
    use crate::err::{LibxrnError, LibxrnResult, XrnErrorKind};
    use xana_commons_rs::XanaError;

    pub fn assert_err_kind(res: LibxrnResult<PathXrn>, expected_kind: XrnErrorKind) {
        match res {
            Ok(res) => panic!("Expected {expected_kind}, got {res}"),
            Err(e) if e.xana_err().kind == expected_kind => {
                // success, we failed!
            }
            Err(e) => panic!("Expected err {expected_kind}, got err {e}"),
        }
    }
}
