use std::net::SocketAddr;

use tokio::net::TcpStream;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use crate::protocol_version::{ProtocolVersion, VERSION_1_2_0};
use crate::{ClientConfiguration, IgniteError};
use crate::ignite_error::{IgniteResult, LogResult};
use crate::ignite_error::ChainResult;
use crate::protocol::message::{HandshakeReq, HandshakeRsp, Response};
use crate::protocol::{OutStream, InStream, Writable, Readable};

/// Versions supported by the client
const SUPPORTED_VERSIONS: [ProtocolVersion; 1] = [VERSION_1_2_0];

/// Represents a single channel to a node of the cluster.
#[derive(Debug)]
pub struct AsyncDataChannel {
    read_end_mutex: Mutex<ReadHalf<TcpStream>>,
    write_end_mutex: Mutex<WriteHalf<TcpStream>>,
    ver: ProtocolVersion,
}

impl AsyncDataChannel {
    /// Try create new data channel between host and the node with a given address.
    pub async fn connect(addr: &SocketAddr, cfg: &ClientConfiguration) -> IgniteResult<Self> {
        debug!("Trying to connect to host: {}", addr);

        let conn_res = tokio::net::TcpStream::connect(&addr).await;
        let conn = conn_res.chain_error(format!("Can not establish connection to host {}", addr))?;

        let (mut read_end, mut write_end) = tokio::io::split(conn);
        let ver = Self::negotiate_version(
            &mut write_end,
            &mut read_end,
            cfg.get_user(),
            cfg.get_password()
        ).await?;

        let read_end_mutex = Mutex::new(read_end);
        let write_end_mutex = Mutex::new(write_end);

        Ok(Self{read_end_mutex, write_end_mutex, ver})
    }

    /// Negotiate protocol version to use.
    async fn negotiate_version(
        write_end: &mut WriteHalf<TcpStream>,
        read_end: &mut ReadHalf<TcpStream>,
        user: &str,
        pwd: &str) -> IgniteResult<ProtocolVersion>
    {
        for ver in SUPPORTED_VERSIONS.iter() {
            let res = Self::handshake(write_end, read_end, ver, user, pwd).await;

            match res {
                Err(_) => res.log_error_w(
                    format!("Can not perform handshake using version {:?}", ver)
                ),
                Ok(()) => return Ok(*ver),
            };
        }

        Err(IgniteError::new("Failed to establish connection to a node using any of supported protocol versions"))
    }

    async fn handshake(
        write_end: &mut WriteHalf<TcpStream>,
        read_end: &mut ReadHalf<TcpStream>,
        ver: &ProtocolVersion,
        user: &str,
        pwd: &str) -> IgniteResult<()>
    {
        Self::handshake_request(write_end, ver, user, pwd).await?;
        Self::handshake_response(read_end).await?;
        Ok(())
    }

    /// Send handshake request using a connection.
    async fn handshake_request(write_end: &mut WriteHalf<TcpStream>, ver: &ProtocolVersion, user: &str, pwd: &str) -> IgniteResult<()> {
        let req = HandshakeReq::new(*ver, user, pwd);
        let data = pack_writable(&req);

        write_end.write_all(&data)
            .await
            .chain_error("Can send handshake request to node".to_owned())?;

        Ok(())
    }

    /// Receive handshake response from a connection.
    async fn handshake_response(read_end: &mut ReadHalf<TcpStream>) -> IgniteResult<()> {
        let data = Self::receive_rsp_raw(read_end).await?;
        let resp = unpack_readable::<HandshakeRsp>(&data);

        match resp {
            Response::Accept(_) => Ok(()),
            Response::Reject(rej) => Err(IgniteError::new("Handshake failed with error: {}")),
        }
    }

    /// Receive response in a raw byte array form
    async fn receive_rsp_raw(read_end: &mut ReadHalf<TcpStream>) -> IgniteResult<Box<[u8]>> {
        use crate::protocol::utils;

        let mut len_buf = [0u8; 4];

        read_end.read_exact(&mut len_buf)
            .await
            .chain_error("Error while reading response length")?;

        let len = utils::deserialize_i32(&len_buf);

        let mut buf = vec![0u8; len as usize].into_boxed_slice();

        read_end.read_exact(&mut buf)
            .await
            .chain_error("Error while reading response payload")?;

        Ok(buf)
    }
}

/// Pack any Writable value into boxed slice.
fn pack_writable(req: &dyn Writable) -> Box<[u8]> {
    let stream = OutStream::new();

    let len = stream.reserve_len();

    req.write(&stream);

    len.set();

    stream.into_memory()
}

/// Pack request with ID into boxed slice.
fn pack_request(req: &dyn Writable, id: i32) -> Box<[u8]> {
    let stream = OutStream::new();

    let len = stream.reserve_len();

    stream.write_i32(id);
    req.write(&stream);

    len.set();

    stream.into_memory()
}

/// Unpack Readable value from slice of bytes.
fn unpack_readable<T: Readable<Item = T>>(data: &[u8]) -> T {
    let stream = InStream::new(data);

    T::read(&stream)
}

// /// Encode and send message to specified stream.
// /// Returns request ID.
// async fn send_message<T>(out: &dyn AsyncWrite, req: T) -> IgniteResult<i32> {
//
// }