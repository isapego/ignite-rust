use crate::protocol::{InStream, Readable};
use crate::protocol::{OutStream, Writable};
use crate::protocol_version::ProtocolVersion;

use super::{RequestType, Response};

/// Type of client. There is only one type of client
/// we are interested in - Thin.
enum ClientType {
    Thin = 2,
}

/// This request is pretty unique as it doesn't implement Request trait.
/// This is because once its issues the protocol connection is not yet
/// established.
pub struct HandshakeReq<'a> {
    ver: ProtocolVersion,
    user: &'a str,
    pass: &'a str,
}

impl<'a> HandshakeReq<'a> {
    /// Make new instance
    pub fn new(ver: ProtocolVersion, user: &'a str, pass: &'a str) -> Self {
        HandshakeReq { ver, user, pass }
    }
}

impl<'a> Writable for HandshakeReq<'a> {
    fn write(&self, out: &OutStream) {
        out.write_i8(RequestType::Handshake as i8);

        self.ver.write(out);

        out.write_i8(ClientType::Thin as i8);

        out.write_str(self.user);
        out.write_str(self.pass);
    }
}

/// Handshake reject. This response is unique just as request, as it does not
/// implement Response trait.
#[allow(dead_code)]
pub struct HandshakeReject {
    ver: ProtocolVersion,
    error: String,
}

impl HandshakeReject {
    /// Make new instance.
    fn new(ver: ProtocolVersion, error: String) -> Self {
        HandshakeReject { ver, error }
    }

    /// Get error
    pub fn get_error(&self) -> &str {
        &self.error
    }
}

/// Handshake response.
pub type HandshakeRsp = Response<(), HandshakeReject>;

impl Readable for HandshakeRsp {
    type Item = HandshakeRsp;

    fn read(stream: &InStream) -> HandshakeRsp {
        let accepted = stream.read_bool();

        if accepted {
            return Response::Accept(());
        }

        let ver = ProtocolVersion::read(stream);
        let err = stream.read_str().unwrap_or_default();

        Response::Reject(HandshakeReject::new(ver, err.into_owned()))
    }
}
