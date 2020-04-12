use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::Encoder;

use crate::IgniteError;

/// A simple `Encoder` implementation for protocol messages.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RequestEncoder(());

impl RequestEncoder {
    /// Creates a new `RequestEncoder` instance.
    pub fn new() -> Self {
        Self(())
    }
}

impl Encoder<Bytes> for RequestEncoder {
    type Error = IgniteError;

    /// Encode request
    fn encode(&mut self, data: Bytes, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.reserve(data.len());
        buf.put(data);
        Ok(())
    }
}
