use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::ignite_error::{ChainResult, IgniteResult, LogResult};
use crate::protocol::message::{HandshakeReq, HandshakeRsp, Response};
use crate::protocol_version::{ProtocolVersion, VERSION_1_2_0};
use crate::{ClientConfiguration, IgniteError};
use crate::protocol::{Writable, Readable, InStream, OutStream};

/// Versions supported by the client
const SUPPORTED_VERSIONS: [ProtocolVersion; 1] = [VERSION_1_2_0];

/// Represents a single channel to a node of the cluster.
#[derive(Debug)]
pub struct DataChannel {
    ver: ProtocolVersion,
    conn: TcpStream,
}

impl DataChannel {
    /// Make new instance
    pub fn new(ver: ProtocolVersion, conn: TcpStream) -> Self {
        Self { ver, conn }
    }

    /// Try create new data channel between host and the node with a given address.
    pub fn connect(addr: &SocketAddr, cfg: &ClientConfiguration) -> IgniteResult<Self> {
        debug!("Trying to connect to host: {}", addr);

        let mut conn = tcp_connect(&addr)
            .chain_error(format!("Can not connect to the host {}", addr))?;

        let ver = negotiate_protocol_version(&mut conn, cfg)?;

        Ok(Self::new(ver, conn))
    }

    /// Send a request and get a response.
    pub fn sync_message<Req, Resp>(&mut self, req: Req) -> IgniteResult<Resp::Item>
    where
        Req: Writable,
        Resp: Readable,
    {
        sync_message_conn::<Req, Resp>(&mut self.conn, req)
    }
}

/// Establish TCP connection with the address
fn tcp_connect(addr: &SocketAddr) -> IgniteResult<TcpStream> {
    let stream = TcpStream::connect(addr)
        .chain_error(format!("Failed to connect to remote host {}", addr))?;

//    stream.set_nonblocking(true).chain_error(format!(
//        "Failed to set connection to non-blocking mode for host {}",
//        addr
//    ))?;

    stream.set_nodelay(true).log_error_w(format!(
        "Failed to set connection to no-delay mode for host {}",
        addr
    ));

    Ok(stream)
}

/// Try to negotiate connection version of a new connection
fn negotiate_protocol_version(
    conn: &mut TcpStream,
    cfg: &ClientConfiguration,
) -> IgniteResult<ProtocolVersion> {
    for ver in SUPPORTED_VERSIONS.iter() {
        let req = HandshakeReq::new(ver.clone(), cfg.get_user(), cfg.get_password());

        let res = sync_message_conn::<HandshakeReq, HandshakeRsp>(conn, req)
            .log_error_w(format!("Handshake failed with version {:?}", ver));

        let resp = match res {
            Some(r) => r,
            None => continue,
        };

        match resp {
            Response::Accept(_) => return Ok(ver.clone()),
            Response::Reject(rej) => warn!("Handshake failed with error: {}", rej.get_error()),
        }
    }

    Err(IgniteError::new(
        "Failed to complete handshake with the host",
    ))
}

/// Receive response in a raw byte array form
fn receive_rsp_raw(conn: &mut TcpStream) -> IgniteResult<Box<[u8]>> {
    use crate::protocol::utils;

    let mut len_buf = [0u8; 4];

    conn.read_exact(&mut len_buf)
        .chain_error("Error while reading response length")?;

    let len = utils::deserialize_i32(&len_buf);

    let mut buf = vec![0u8; len as usize].into_boxed_slice();

    conn.read_exact(&mut buf)
        .chain_error("Error while reading response payload")?;

    Ok(buf)
}

/// Send a request and get a response.
fn sync_message_conn<Req, Resp>(conn: &mut TcpStream, req: Req) -> IgniteResult<Resp::Item>
    where
        Req: Writable,
        Resp: Readable,
{
    let req_data = pack_writable(&req);

    conn.write_all(&req_data)
        .chain_error("Can not send request")?;

    let rsp_data = receive_rsp_raw(conn).chain_error("Can not receive response")?;

    Ok(unpack_readable::<Resp::Item, Resp>(&rsp_data))
}

fn unpack_readable<I, T: Readable<Item = I>>(data: &[u8]) -> I {
    let stream = InStream::new(data);

    T::read(&stream)
}

/// Pack any Writable value into boxed slice
fn pack_writable(req: &dyn Writable) -> Box<[u8]> {
    let stream = OutStream::new();

    let len = stream.reserve_len();

    req.write(&stream);

    len.set();

    stream.into_memory()
}
