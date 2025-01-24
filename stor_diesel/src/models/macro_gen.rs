use crate::err::StorDieselError;

/// Handy to convert native SQL as raw text
/// into user-friendly Enums and chrono DateTimes
#[macro_export]
macro_rules! gen_try_from_converter {
    ($struct_raw: ident, $struct_sql: ident, ( $($field_name_only: ident),* ), $( ($field_name: ident, $convert: expr), )+ ) => {
        impl TryFrom<$struct_raw> for $struct_sql {
            type Error = StorDieselError;
            fn try_from(value: $struct_raw) -> Result<Self, Self::Error> {

                Ok($struct_sql {
                    $( $field_name_only: value.$field_name_only, )*
                    $( $field_name: crate::models::macro_gen::ConvertWrap($convert(value.$field_name)).aconvert()?, )+

                })
            }
        }
    };
}

/// Conversions can return either the value or a Result with the value
/// so the macro can always use ?
pub struct ConvertWrap<T>(pub T);

impl<R, E> ConvertWrap<Result<R, E>>
where
    StorDieselError: From<E>,
{
    pub fn aconvert(self) -> Result<R, StorDieselError> {
        self.0.map_err(Into::into)
    }
}

impl ConvertWrap<String> {
    pub fn aconvert(self) -> Result<String, StorDieselError> {
        Ok(self.0)
    }
}
