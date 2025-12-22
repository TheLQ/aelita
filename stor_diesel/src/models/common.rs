use crate::StorDieselError;
use crate::err::{StorDieselErrorKind, StorDieselErrorMeta};
use std::str::FromStr;

pub fn parse_type_checked<Enum>(bytes: &[u8]) -> Result<Enum, Box<StorDieselError>>
where
    Enum: FromStr<Err = strum::ParseError>,
{
    let input = str::from_utf8(bytes).map_err(|e| StorDieselErrorMeta::StdUtf8(e).build())?;
    Enum::from_str(input).map_err(|e| StorDieselErrorMeta::Strum(e).build())
}
