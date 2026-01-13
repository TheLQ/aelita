use std::fmt::Display;
use std::marker::PhantomData;
use xana_commons_rs::LOCALE;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::trace;

/// Before this: How much is left?
/// ```text
/// Doing something
/// Doing something
/// ...
/// Doing something
/// ...
/// ```
pub struct Chunky<T, M: Display> {
    input: T,
    message: M,
}

impl<T, M: Display> Chunky<T, M> {
    pub fn ify(input: T, message: M) -> Self {
        Self { input, message }
    }

    pub fn ify_as_ref<'c, InnerNeedsConv, Inner>(
        input: &'c T,
        message: M,
    ) -> Chunky<ChunkyAsRef<'c, InnerNeedsConv, Inner>, M>
    where
        T: AsRef<[InnerNeedsConv]>,
        InnerNeedsConv: AsRef<[Inner]>,
    {
        Chunky::ify(ChunkyAsRef(input.as_ref(), PhantomData), message)
        // Chunky::ify(input, message)
    }
}

impl<T, M: Display> Chunky<T, M>
where
    Self: ChunkyPiece,
{
    fn log_passthru(
        chunks_len: usize,
        message: M,
    ) -> impl FnMut((usize, <Self as ChunkyPiece>::Value)) -> <Self as ChunkyPiece>::Value {
        move |(i, value)| {
            trace!(
                "Chunky {message} - {} of {}",
                i.to_formatted_string(&LOCALE),
                chunks_len.to_formatted_string(&LOCALE)
            );
            value
        }
    }
}

pub trait ChunkyPiece {
    type Value;

    fn pieces<const SIZE: usize>(self) -> impl Iterator<Item = Self::Value>;
}

impl<T, M: Display> ChunkyPiece for Chunky<Vec<T>, M> {
    type Value = Box<[T]>;

    fn pieces<const SIZE: usize>(self) -> impl Iterator<Item = Self::Value> {
        let Self { mut input, message } = self;

        // why into_chunks why
        // - truncates remainder, so we need to save it first
        // - Remainder Vec and Fixed Array are converted to Boxed Slice

        let input_len = input.len();
        let remainder = input_len % SIZE;
        let remainder = if remainder != 0 {
            input
                .drain((input_len - remainder)..)
                .collect::<Vec<_>>()
                .into_boxed_slice()
        } else {
            Box::new([])
        };
        assert_eq!(input.len() % SIZE, 0);

        let chunks_len = chunks_in_len(SIZE, &input);
        input
            .into_chunks::<SIZE>()
            .into_iter()
            .map(|v| {
                let new: Box<[T]> = Box::new(v);
                new
            })
            .chain([remainder].into_iter())
            .enumerate()
            .map(Self::log_passthru(chunks_len, message))
    }
}

impl<'t, T, M: Display> ChunkyPiece for Chunky<&'t [T], M> {
    type Value = &'t [T];

    fn pieces<const SIZE: usize>(self) -> impl Iterator<Item = Self::Value> {
        let Self { input, message } = self;
        let chunks_len = chunks_in_len(SIZE, input);
        input
            .chunks(SIZE)
            .enumerate()
            .map(Self::log_passthru(chunks_len, message))
    }
}

struct ContainerWithLifetime<'a, T: 'a>(T, PhantomData<&'a T>);

pub struct ChunkyAsRef<'container, InnerNeedsConv, Inner>(
    &'container [InnerNeedsConv],
    PhantomData<Inner>,
)
where
    InnerNeedsConv: AsRef<[Inner]>;

impl<'container, InnerNeedsConv, Inner> ChunkyAsRef<'container, InnerNeedsConv, Inner>
where
    InnerNeedsConv: AsRef<[Inner]>,
{
    pub fn new(input: &'container [InnerNeedsConv]) -> Self {
        Self(input, PhantomData)
    }
}

impl<'c, InnerNeedsConv, Inner, M: Display> ChunkyPiece
    for Chunky<ChunkyAsRef<'c, InnerNeedsConv, Inner>, M>
where
    InnerNeedsConv: AsRef<[Inner]> + 'c,
    Inner: 'c,
{
    type Value = Vec<&'c [Inner]>;

    fn pieces<const SIZE: usize>(self) -> impl Iterator<Item = Self::Value> {
        let Self { input, message } = self;
        let input = input.0.as_ref();

        // why into_chunks why
        // - truncates remainder, so we need to save it first
        // - Remainder Vec and Fixed Array are converted to Boxed Slice

        let input_len = input.len();
        let remainder = input_len % SIZE;
        let remainder = if remainder != 0 {
            vec![&input[(input_len - remainder)..]]
        } else {
            Vec::new()
        };
        assert_eq!(input.len() % SIZE, 0);

        let chunks = input.chunks(SIZE).collect::<Vec<_>>();

        let chunks_len = chunks_in_len(SIZE, &input);
        input
            .chunks(SIZE)
            .chain(remainder.into_iter())
            .map(|needs_conv_chunk| {
                needs_conv_chunk
                    .iter()
                    .map(|v| v.as_ref())
                    .collect::<Vec<&[Inner]>>()
            })
            .enumerate()
            .map(Self::log_passthru(chunks_len, message))
    }
}

fn chunks_in_len<T>(chunk_size: usize, slice: &[T]) -> usize {
    let len = slice.len();
    let chunks = len / chunk_size;
    let remainder = len % chunk_size;
    if remainder == 0 { chunks } else { chunks + 1 }
}
