use crate::defs::address::{XrnAddr, XrnAddrRef, XrnType};
use crate::err::{LibxrnResult, XrnErrorKind};
use xana_commons_rs::CrashErrKind;
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

pub trait XrnSubTypeImpl: XrnTypeImpl {}

pub trait SubXrnImpl: XrnAddrRef {
    const UPPER: XrnType;
    type SubXrnType: XrnSubTypeImpl;

    fn sub_type(&self) -> Self::SubXrnType;
}

pub fn check_expected_type(upper: XrnType, addr: &XrnAddr) -> LibxrnResult<()> {
    if addr.merge().to_type() != upper {
        Err(XrnErrorKind::UnexpectedType.build_message(format!("expected {upper} got {addr}")))
    } else {
        Ok(())
    }
}
