use crate::ignite_error::IgniteResult;

// Trait for a type that can be read from a stream
pub trait Read {
    type Item: Sized;

    fn read(stream: &mut InStream) -> Self::Item;
}

pub struct InStream<'a> {
    mem: &'a [u8],
}

