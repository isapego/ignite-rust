use bytes::{Bytes, BufMut, BytesMut};
use tokio_util::codec::{Encoder, Decoder};

use crate::IgniteError;

/// A simple `Decoder` implementation for protocol messages.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ResponseDecoder(());

impl ResponseDecoder {
    /// Creates a new `ResponseDecoder` instance.
    pub fn new() -> Self { Self(())  }
}

impl Decoder for ResponseDecoder {
    type Item = BytesMut;
    type Error = IgniteError;

    /// Decode response
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.len() > 0 {
            let len = buf.len();
            Ok(Some(buf.split_to(len)))
        } else {
            Ok(None)
        }
    }
}
