// Trait for a type that can be read from a stream
pub trait Readable {
    type Item: Sized;

    fn read(stream: &InStream) -> Self::Item;
}

pub struct InStream<'a> {
    mem: &'a [u8],
}
