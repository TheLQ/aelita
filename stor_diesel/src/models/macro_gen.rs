use crate::err::StorDieselError;

#[macro_export]
macro_rules! gen_try_from_converter {
    ($struct_raw: ident, $struct_sql: ident, ( $($field_name_only: ident),+ ), $( ($field_name: ident, $convert: expr), )+ ) => {
        impl TryFrom<$struct_raw> for $struct_sql {
            type Error = StorDieselError;
            fn try_from(value: $struct_raw) -> Result<Self, Self::Error> {

                Ok($struct_sql {
                    $( $field_name_only: value.$field_name_only, )+
                    $( $field_name: ConvertWrap($convert(value.$field_name)).aconvert()?, )+

                })
            }
        }
    };
}

/// Conversions can return either the value or a Result with the value
struct ConvertWrap<T>(T);

impl<R, E> ConvertWrap<Result<R, E>>
where
    StorDieselError: From<E>,
{
    fn aconvert(self) -> Result<R, StorDieselError> {
        self.0.map_err(Into::into)
    }
}

impl ConvertWrap<String> {
    fn aconvert(self) -> Result<String, StorDieselError> {
        Ok(self.0)
    }
}
