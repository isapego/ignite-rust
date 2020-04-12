use crate::protocol::OutStream;
use crate::protocol_version::ProtocolVersion;

use crate::IgniteError;

/// Type of request message
pub enum RequestType {
    Handshake = 1,
    CacheCreateWithName = 1051,
}

/// Trait for a type representing protocol request message
pub trait Request {
    /// Type of the request.
    const TYPE: RequestType;

    /// Type of response if the request was accepted.
    type Response;

    /// Write payload of the request message.
    fn write_payload(&self, out: &mut OutStream, ver: &ProtocolVersion);
}

/// Response enum.
pub enum Response<A, R> {
    Accept(A),
    Reject(R),
}

/// Simple response type used in the most of cases.
pub type SimpleResponse<A> = Response<A, GeneralResponseReject>;

/// General response reject.
#[allow(dead_code)]
pub struct GeneralResponseReject {
    status: i32,
    error: String,
}

impl GeneralResponseReject {
    /// Get status
    #[allow(dead_code)]
    pub fn status(&self) -> i32 {
        self.status
    }

    /// Get error
    #[allow(dead_code)]
    pub fn error(&self) -> &str {
        &self.error
    }

    /// Move error message out of 
    #[allow(dead_code)]
    pub fn decompose(self) -> (i32, String) {
        (self.status, self.error)
    }
}

impl Into<IgniteError> for GeneralResponseReject {
    fn into(self) -> IgniteError {
        IgniteError::new(self.error)
    }
}
