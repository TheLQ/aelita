use crate::defs::address::{XrnAddr, XrnType};
use xana_commons_rs::tracing_re::trace;

pub trait XrnTypeImpl
where
    Self: strum::VariantArray + Clone + AsRef<str>,
{
    fn is_starts_with<'s>(&'static self, input: &'s str) -> Option<(Self, &'s str)> {
        let lookup: &'static str = self.as_ref();
        trace!("testing {lookup} on {input}");

        let (sep, remain) = input.split_at_checked(1)?;
        assert_eq!(sep, ":");
        if sep != ":" {
            return None;
        }

        let (xtype, remain) = remain.split_at_checked(lookup.len())?;
        if xtype == lookup {
            Some((self.clone(), remain))
        } else {
            None
        }
    }

    fn split_type(value: &str) -> Option<(Self, &str)> {
        Self::VARIANTS
            .iter()
            .filter_map(|ty| ty.is_starts_with(value))
            .next()
    }
}
