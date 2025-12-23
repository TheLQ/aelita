use crate::StorDieselError;
use crate::err::{StorDieselErrorKind, StorDieselErrorMeta};
use std::str::FromStr;

pub fn parse_type_checked<Enum>(bytes: &[u8]) -> Result<Enum, Box<StorDieselError>>
where
    Enum: FromStr<Err = strum::ParseError>,
{
    let input = str::from_utf8(bytes)?;
    Enum::from_str(input).map_err(Into::into)
}
