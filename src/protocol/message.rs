use crate::protocol::{OutStream, Write};
use crate::protocol_version::ProtocolVersion;

/// Type of client. There is only one type of client
/// we are interested in - Thin.
enum ClientType {
    Thin = 2,
}

/// Type of request message
enum RequestType {
    Handshake = 1,
}

/// Type of response message
pub enum ResponseType {
    Handshake = 1,
}

/// Trait for a type representing protocol request message
pub trait Request {
    fn write(&self, out: &mut OutStream, ver: &ProtocolVersion);
}

/// This request is pretty unique as it doesn't implement Request trait.
/// This is because once its issues the protocol connection is not yet
/// established, and no ProtocolVersion is known.
pub struct HandshakeReq<'a> {
    ver: ProtocolVersion,
    user: &'a str,
    pass: &'a str,
}

impl<'a> Write for HandshakeReq<'a> {
    fn write(&self, out: &OutStream) {
        out.write_i8(RequestType::Handshake as i8);

        self.ver.write(out);

        out.write_i8(ClientType::Thin as i8);

        out.write_str(self.user);
        out.write_str(self.pass);
    }
}

impl<'a> HandshakeReq<'a> {
    /// Make new instance
    pub fn new(ver: ProtocolVersion, user: &'a str, pass: &'a str) -> Self {
        HandshakeReq {
            ver: ver,
            user: user,
            pass: pass,
        }
    }
}
