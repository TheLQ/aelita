use crate::defs::address::{XrnAddr, XrnType};
use xana_commons_rs::tracing_re::trace;

pub trait SubXrn {
    fn atype() -> XrnType;

    fn to_addr<'u>(&'u self) -> XrnAddr
    where
        XrnAddr: From<&'u Self>,
    {
        self.into()
    }
}

pub trait XrnTypeImpl<'s>
where
    Self: strum::VariantArray + Clone + AsRef<str>,
{
    fn is_starts_with(&'static self, input: &'s str) -> Option<(Self, &'s str)> {
        let lookup: &'static str = self.as_ref();
        trace!("testing {lookup} on {input}");

        let (sep, remain) = input.split_at_checked(1)?;
        assert_eq!(sep, ":");
        if sep != ":" {
            return None;
        }

        let (xtype, remain) = remain.split_at_checked(lookup.len())?;
        if xtype != lookup {
            None
        } else {
            Some((self.clone(), remain))
        }
    }

    fn split_type(value: &'s str) -> Option<(Self, &'s str)> {
        Self::VARIANTS
            .iter()
            .filter_map(|ty| ty.is_starts_with(value))
            .next()
    }
}
