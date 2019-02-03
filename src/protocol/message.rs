use crate::protocol::{InStream, Readable};
use crate::protocol::{OutStream, Writable};
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
enum ResponseType {
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

impl<'a> Writable for HandshakeReq<'a> {
    fn write(&self, out: &OutStream) {
        out.write_i8(RequestType::Handshake as i8);

        self.ver.write(out);

        out.write_i8(ClientType::Thin as i8);

        out.write_str(self.user);
        out.write_str(self.pass);
    }
}

/// Response enum.
pub enum Response<A, R> {
    Accept(A),
    Reject(R),
}

/// Handshake reject. This response is unique just as request, as it does not
/// implement Response trait.
pub struct HandshakeReject {
    ver: ProtocolVersion,
    error: String,
}

impl HandshakeReject {
    /// Make new instance.
    fn new(ver: ProtocolVersion, error: String) -> Self {
        HandshakeReject {
            ver: ver,
            error: error,
        }
    }
}

/// Handshake response.
pub type HandshakeRsp = Response<(), HandshakeReject>;

// impl Readable for HandshakeRsp {
//     type Item = HandshakeRsp;

//     fn read(stream: &InStream) -> HandshakeRsp {
//         let accepted = stream.read_bool();

//         if accepted {
//             return Response::Accept(());
//         }

//         let ver = ProtocolVersion::read(stream);
//         let err = stream.read_str();

//         Response::Reject(HandshakeReject::new(ver, err))
//     }
// }
