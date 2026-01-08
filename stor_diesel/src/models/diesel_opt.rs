use diesel::{Queryable, backend::Backend, deserialize};

/// source https://github.com/diesel-rs/diesel/discussions/3835#discussioncomment-8946707
pub struct OptTryInto<T>(Option<T>);

impl<ST, DB, T> Queryable<ST, DB> for OptTryInto<T>
where
    Option<T>: Queryable<ST, DB>,
    DB: Backend,
{
    type Row = <Option<T> as Queryable<ST, DB>>::Row;

    fn build(row: Self::Row) -> deserialize::Result<Self> {
        Ok(Self(Option::<T>::build(row)?))
    }
}

impl<T> OptTryInto<T> {
    // This is a hack based on how the queryable derive works (by calling `.try_into()`)
    pub fn try_into<O, E>(self) -> diesel::deserialize::Result<Option<O>>
    where
        T: TryInto<O, Error = E>,
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        self.0
            .map(TryInto::try_into)
            .transpose()
            .map_err(Into::into)
    }
}

#[cfg(test)]
#[allow(unused)]
mod test {
    use super::*;

    #[derive(diesel::Queryable)]
    struct Inner {
        #[diesel(deserialize_as = OptTryInto<i32>)]
        value: Option<u32>,
    }
}
