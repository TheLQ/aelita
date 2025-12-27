use crate::err::StorDieselErrorKind;
use crate::StorDieselResult;
use std::str::FromStr;
use xana_commons_rs::CrashErrKind;

pub fn parse_type_checked<Enum>(bytes: &[u8]) -> StorDieselResult<Enum>
where
    Enum: FromStr<Err = strum::ParseError>,
{
    let input = str::from_utf8(bytes).map_err(StorDieselErrorKind::UnknownVariant.err_map())?;
    Enum::from_str(input).map_err(StorDieselErrorKind::UnknownVariant.err_map())
}
