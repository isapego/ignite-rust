use std::net::SocketAddr;

use tokio::net::TcpStream;
// use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::Mutex;
use tokio_util::codec::{FramedWrite, FramedRead};

use futures::{AsyncWrite, AsyncRead};

use crate::protocol_version::{ProtocolVersion, VERSION_1_2_0};
use crate::{IgniteError, ClientConfiguration};
use crate::ignite_error::{ChainResult, IgniteResult, LogResult};

/// Versions supported by the client
const SUPPORTED_VERSIONS: [ProtocolVersion; 1] = [VERSION_1_2_0];

/// Represents a single channel to a node of the cluster.
pub struct AsyncDataChannel {
    read_end_mutex: ReadHalf<TcpStream>,
    write_end_mutex: WriteHalf<TcpStream>,
    ver: ProtocolVersion,
}

impl AsyncDataChannel {
    /// Try create new data channel between host and the node with a given address.
    pub async fn connect(ver: ProtocolVersion, addr: SocketAddr, cfg: &ClientConfiguration) -> IgniteResult<Self> {
        debug!("Trying to connect to host: {}", addr);

        let conn_res = tokio::net::TcpStream::connect(&addr).await;
        let mut conn = conn_res.chain_error(format!("Can not establish connection to host {}", addr))?;

        let (read_end, write_end) = tokio::io::split(conn);
        // let read_end_mutex = Mutex::new(read_end);
        // let write_end_mutex = Mutex::new(write_end);

        // Self::handshake(&mut conn, &ver, cfg.get_user(), cfg.get_password()).await?;

        Ok(Self::new(read_end, write_end, VERSION_1_2_0))
    }

    fn new(read_end_mutex: ReadHalf<TcpStream>,
           write_end_mutex: WriteHalf<TcpStream>,
           ver: ProtocolVersion) -> Self {
        Self{read_end_mutex, write_end_mutex, ver}
    }

//     /// Perform handshake on a connection.
//     async fn handshake(conn: &mut TcpStream, ver: &ProtocolVersion, user: &str, pwd: &str) -> IgniteResult<()> {
// //        let stream = WriteBuf::with_fn();
//
// //        conn.write_all(data).await?;
//
//         unimplemented!()
//     }

//    async fn raw_message(&self, message: &[u8]) -> Vec<u8> {
//
//    }
}