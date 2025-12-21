use crate::StorDieselError;
use std::fmt::Display;
use std::str::FromStr;

pub fn parse_type_checked<Enum>(bytes: &[u8]) -> Result<Enum, Box<StorDieselError>>
where
    Enum: FromStr,
    <Enum as FromStr>::Err: Display,
{
    let input = str::from_utf8(bytes).map_err(|e| {
        Box::new(StorDieselError::query_fail(format!(
            "variant not bytes {e}"
        )))
    })?;
    Enum::from_str(input).map_err(|e| {
        Box::new(StorDieselError::query_fail(format!(
            "unsupported variant {e}"
        )))
    })
}
