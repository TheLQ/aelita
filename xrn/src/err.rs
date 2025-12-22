pub type LibxrnResult<T> = Result<T, Box<LibxrnError>>;

#[derive(Debug, PartialEq, Eq, strum::AsRefStr, strum::Display)]
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

#[cfg(test)]
pub mod test {
    use crate::defs::path_xrn::PathXrn;
    use crate::err::{LibxrnError, LibxrnErrorMeta, LibxrnResult, XrnErrorKind};

    pub fn assert_err_kind(res: LibxrnResult<PathXrn>, expected_kind: XrnErrorKind) {
        match res {
            Ok(res) => panic!("Expected {expected_kind}, got {res}"),
            Err(e) if matches!(std::ops::Deref::deref(&e), LibxrnError { meta: LibxrnErrorMeta::LibxrnError(kind), .. } if *kind == expected_kind) =>
            {
                // success, we failed!
            }
            Err(e) => panic!("Expected err {expected_kind}, got err {e}"),
        }
    }
}
