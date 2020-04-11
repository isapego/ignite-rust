use std::net::SocketAddr;

use tokio::net::TcpStream;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use crate::protocol_version::{ProtocolVersion, VERSION_1_2_0};
use crate::{ClientConfiguration, IgniteError};
use crate::ignite_error::IgniteResult;
use crate::ignite_error::ChainResult;
use crate::protocol::message::{HandshakeReq, HandshakeRsp, Response};
use crate::protocol::{OutStream, InStream, Writable, Readable};

/// Versions supported by the client
const SUPPORTED_VERSIONS: [ProtocolVersion; 1] = [VERSION_1_2_0];

/// Represents a single channel to a node of the cluster.
pub struct AsyncDataChannel {
    read_end_mutex: Mutex<ReadHalf<TcpStream>>,
    write_end_mutex: Mutex<WriteHalf<TcpStream>>,
    ver: ProtocolVersion,
}

impl AsyncDataChannel {
    /// Try create new data channel between host and the node with a given address.
    pub async fn connect(ver: ProtocolVersion, addr: SocketAddr, cfg: &ClientConfiguration) -> IgniteResult<Self> {
        debug!("Trying to connect to host: {}", addr);

        let conn_res = tokio::net::TcpStream::connect(&addr).await;
        let conn = conn_res.chain_error(format!("Can not establish connection to host {}", addr))?;

        let (mut read_end, mut write_end) = tokio::io::split(conn);
        Self::handshake_request(&mut write_end, &ver, cfg.get_user(), cfg.get_password()).await?;
        Self::handshake_response(&mut read_end).await?;

        let read_end_mutex = Mutex::new(read_end);
        let write_end_mutex = Mutex::new(write_end);

        Ok(Self{read_end_mutex, write_end_mutex, ver})
    }

    /// Send handshake request using a connection.
    async fn handshake_request(conn: &mut WriteHalf<TcpStream>, ver: &ProtocolVersion, user: &str, pwd: &str) -> IgniteResult<()> {
        let req = HandshakeReq::new(*ver, user, pwd);
        let data = pack_writable(&req);

        conn.write_all(&data)
            .await
            .chain_error("Can send handshake request to node".to_owned())?;

        Ok(())
    }

    /// Receive handshake response from a connection.
    async fn handshake_response(conn: &mut ReadHalf<TcpStream>) -> IgniteResult<()> {
        let data = Self::receive_rsp_raw(conn).await?;
        let resp = unpack_readable::<HandshakeRsp>(&data);

        match resp {
            Response::Accept(_) => Ok(()),
            Response::Reject(rej) => Err(IgniteError::new("Handshake failed with error: {}")),
        }
    }

    /// Receive response in a raw byte array form
    async fn receive_rsp_raw(conn: &mut ReadHalf<TcpStream>) -> IgniteResult<Box<[u8]>> {
        use crate::protocol::utils;

        let mut len_buf = [0u8; 4];

        conn.read_exact(&mut len_buf)
            .await
            .chain_error("Error while reading response length")?;

        let len = utils::deserialize_i32(&len_buf);

        let mut buf = vec![0u8; len as usize].into_boxed_slice();

        conn.read_exact(&mut buf)
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