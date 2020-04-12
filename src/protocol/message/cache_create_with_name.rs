use crate::protocol::OutStream;
use crate::protocol_version::ProtocolVersion;

use super::common::*;

/// Request sent when creating cache with name.
pub struct CacheCreateWithNameReq<'a> {
    cache_name: &'a str,
}

impl<'a> CacheCreateWithNameReq<'a> {
    /// Create new instance of the request.
    #[allow(dead_code)]
    pub fn new(cache_name: &'a str) -> Self {
        Self{cache_name}
    }
}

impl <'a> Request for CacheCreateWithNameReq<'a> {
    /// Request type.
    const TYPE: RequestType = RequestType::CacheCreateWithName;

    /// Response type.
    type Response = SimpleResponse<()>;

    /// Write payload of the request message.
    fn write_payload(&self, out: &mut OutStream, _ver: &ProtocolVersion) {
        out.write_str(self.cache_name);
    }
}