use crate::protocol::{OutStream, Write};
use crate::protocol_version::ProtocolVersion;

enum ClientType {
    Thin = 2,
}

enum RequestType {
    Handshake = 1,
}

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
        let len = out.reserve_i32();

        out.write_i8(RequestType::Handshake as i8);

        self.ver.write(out);

        out.write_i8(ClientType::Thin as i8);

        out.write_str(self.user);
        out.write_str(self.pass);

        len.set((out.position() - 4) as i32);
    }
}
