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
}

impl<T, M: Display> Chunky<T, M> where Self: ChunkyPiece {}

pub trait ChunkyPiece {
    type Value;

    fn pieces<const SIZE: usize>(self) -> impl Iterator<Item = Self::Value>;

    fn log_passthru<M: Display>(
        chunks_len: usize,
        total: usize,
        message: M,
    ) -> impl FnMut((usize, Self::Value)) -> Self::Value {
        move |(i, value)| {
            trace!(
                "Chunky {message} - {} of {} total_len {total}",
                i.to_formatted_string(&LOCALE),
                (chunks_len.checked_sub(1).unwrap_or(666_666_666)).to_formatted_string(&LOCALE)
            );
            value
        }
    }
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
            .map(Self::log_passthru(chunks_len, input_len, message))
    }
}

impl<'t, T, M: Display> ChunkyPiece for Chunky<&'t [T], M> {
    type Value = &'t [T];

    fn pieces<const SIZE: usize>(self) -> impl Iterator<Item = Self::Value> {
        let Self { input, message } = self;
        let chunks_len = chunks_in_len(SIZE, input);
        let input_len = input.len();
        input
            .chunks(SIZE)
            .enumerate()
            .map(Self::log_passthru(chunks_len, input_len, message))
    }
}

pub struct ChunkyAsRef<'container, InnerNeedsConv, Inner, M: Display>(
    Chunky<&'container [InnerNeedsConv], M>,
    PhantomData<Inner>,
)
where
    InnerNeedsConv: AsRef<[Inner]>;

impl<'container, InnerNeedsConv, Inner, M: Display>
    ChunkyAsRef<'container, InnerNeedsConv, Inner, M>
where
    InnerNeedsConv: AsRef<[Inner]>,
{
    pub fn new(input: &'container [InnerNeedsConv], message: M) -> Self {
        Self(Chunky { input, message }, PhantomData)
    }
}

impl<'container, InnerNeedsConv, Inner, M: Display> ChunkyPiece
    for ChunkyAsRef<'container, InnerNeedsConv, Inner, M>
where
    InnerNeedsConv: AsRef<[Inner]> + 'container,
    Inner: 'container,
{
    type Value = Vec<&'container [Inner]>;

    fn pieces<const SIZE: usize>(self) -> impl Iterator<Item = Self::Value> {
        let Chunky { input, message } = self.0;
        let input_len = input.len();
        let chunks_len = chunks_in_len(SIZE, &input);
        input
            .chunks(SIZE)
            .map(|needs_conv_chunk| {
                needs_conv_chunk
                    .iter()
                    .map(|v| v.as_ref())
                    .collect::<Vec<&[Inner]>>()
            })
            .enumerate()
            .map(Self::log_passthru(chunks_len, input_len, message))
    }
}

fn chunks_in_len<T>(chunk_size: usize, slice: &[T]) -> usize {
    let len = slice.len();
    let chunks = len / chunk_size;
    let remainder = len % chunk_size;
    if remainder == 0 { chunks } else { chunks + 1 }
}
