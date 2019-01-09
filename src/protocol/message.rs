use protocol_version::ProtocolVersion;

enum ClientType {
    Thin = 2,
}

enum RequestType {
    Handshake = 1,
}

pub enum ResponseType {
    Handshake = 1,
}

pub struct HandshakeReq<'a> {
    ver: ProtocolVersion,
    user: &'a str,
    pass: &'a str,
}
