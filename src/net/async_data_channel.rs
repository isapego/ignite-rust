use std::net::SocketAddr;

use tokio::net::TcpStream;
// use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
// use tokio_util::codec::{FramedWrite, FramedRead};

// use futures::{AsyncWrite, AsyncRead};

use crate::protocol_version::{ProtocolVersion, VERSION_1_2_0};
// use crate::IgniteError;
use crate::ClientConfiguration;
use crate::ignite_error::IgniteResult;
use crate::ignite_error::ChainResult;
use crate::protocol::message::HandshakeReq;
use crate::protocol::{OutStream, Writable};
// use crate::ignite_error::LogResult;

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
        let mut conn = conn_res.chain_error(format!("Can not establish connection to host {}", addr))?;

        Self::handshake(&mut conn, &ver, cfg.get_user(), cfg.get_password()).await?;

        let (read_end, write_end) = tokio::io::split(conn);
        let read_end_mutex = Mutex::new(read_end);
        let write_end_mutex = Mutex::new(write_end);

        Ok(Self{read_end_mutex, write_end_mutex, ver})
    }

    /// Perform handshake on a connection.
    async fn handshake(conn: &mut TcpStream, ver: &ProtocolVersion, user: &str, pwd: &str) -> IgniteResult<()> {
        let req = HandshakeReq::new(*ver, user, pwd);
        let data = pack_writable(&req);

        conn.write_all(&data).await.chain_error("Can send handshake request to node")?;

        unimplemented!();
    }
}

/// Pack any Writable value into boxed slice
fn pack_writable(req: &dyn Writable) -> Box<[u8]> {
    let stream = OutStream::new();

    let len = stream.reserve_len();

    req.write(&stream);

    len.set();

    stream.into_memory()
}

/// Pack request with ID into boxed slice
fn pack_request(req: &dyn Writable, id: i32) -> Box<[u8]> {
    let stream = OutStream::new();

    let len = stream.reserve_len();

    stream.write_i32(id);
    req.write(&stream);

    len.set();

    stream.into_memory()
}


// /// Encode and send message to specified stream.
// /// Returns request ID.
// async fn send_message<T>(out: &dyn AsyncWrite, req: T) -> IgniteResult<i32> {
//
// }