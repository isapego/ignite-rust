extern crate paste;

use super::header;
use super::{InStream, OutStream};

/// Represents basic protocol type and defines a set of operations over it
pub trait ProtocolType {
    type Item;
    const HEADER: i8;

    fn write_payload(&self, stream: &OutStream);
    fn read_payload(stream: &InStream) -> Self::Item;
}

/// Write full value
#[allow(dead_code)]
pub fn write_full<T, I>(val: T, stream: &OutStream)
where
    T: ProtocolType<Item = I>,
{
    stream.write_i8(T::HEADER);
    val.write_payload(stream);
}

/// Read full value
#[allow(dead_code)]
pub fn read_full<T, I>(stream: &InStream) -> Option<I>
where
    T: ProtocolType<Item = I>,
{
    let header = stream.read_i8();

    if header == T::HEADER {
        Some(T::read_payload(stream))
    } else if header == header::NULL {
        None
    } else {
        panic!("Header is not expected: {}", header);
    }
}

macro_rules! impl_proto_for_primitive {
    ($ttype:ty, $header:expr) => {
        impl ProtocolType for $ttype {
            type Item = $ttype;
            const HEADER: i8 = $header;

            paste::item! {
                fn write_payload(&self, stream: &OutStream) {
                    stream. [<write_ $ttype>] (*self);
                }
            }

            paste::item! {
                fn read_payload(stream: &InStream) -> Self::Item {
                    stream. [<read_ $ttype>] ()
                }
            }
        }
    };
}

impl_proto_for_primitive!(i8, header::BYTE);
impl_proto_for_primitive!(i16, header::SHORT);
impl_proto_for_primitive!(i32, header::INT);
impl_proto_for_primitive!(i64, header::LONG);

impl<'a> ProtocolType for &'a str {
    type Item = String;
    const HEADER: i8 = header::STRING;

    fn write_payload(&self, stream: &OutStream) {
        stream.write_str_raw(*self);
    }

    fn read_payload(stream: &InStream) -> Self::Item {
        stream.read_str_raw().into()
    }
}
