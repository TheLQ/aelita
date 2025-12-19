use crate::defs::address::XrnType;
use xana_commons_rs::tracing_re::trace;

pub trait SubXrn {
    fn atype() -> XrnType;
}

pub trait XrnTypeImpl<'s>
where
    Self: strum::VariantArray + Clone,
    &'static str: From<&'s Self>,
{
    fn is_starts_with(&'static self, input: &'s str) -> Option<(Self, &'s str)> {
        let lookup: &'static str = self.into();
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
