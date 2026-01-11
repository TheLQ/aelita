use std::fmt::{Display, Formatter};

pub struct DisplayCompPath<'t, T>(pub &'t [T]);

impl<Inner> Display for DisplayCompPath<'_, Inner>
where
    Inner: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for comp in self.0.as_ref() {
            let prefix = if !is_first { "/" } else { "" };
            write!(
                f,
                "{prefix}{}",
                str::from_utf8(comp.as_ref()).unwrap_or("INVALID_UTF8")
            )?;
            is_first = false;
        }
        Ok(())
    }
}
