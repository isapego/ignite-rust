extern crate futures;
extern crate tokio;

// use std::env;
use std::net::SocketAddr;

// use futures::Future;
// use futures::stream::Stream;
//use tokio_io::AsyncRead;
//use tokio_io::io::copy;
use tokio::net::TcpStream;
//use tokio_core::reactor::Core;

use crate::protocol_version::ProtocolVersion;
use crate::{IgniteError, ClientConfiguration};
use crate::ignite_error::{ChainResult, IgniteResult, LogResult};


/// Represents a single channel to a node of the cluster.
#[derive(Debug)]
pub struct AsyncDataChannel {
    ver: ProtocolVersion,
    conn: TcpStream,
}

impl AsyncDataChannel {
    /// Try create new data channel between host and the node with a given address.
    pub async fn connect(ver: ProtocolVersion, addr: SocketAddr, cfg: &ClientConfiguration) -> IgniteResult<Self> {
        debug!("Trying to connect to host: {}", addr);

        let conn_res = tokio::net::TcpStream::connect(&addr).await;

        let conn = conn_res.chain_error(format!("Can not establish connection to host {}", addr))?;



//            .map(|c| {
//                c.set_nodelay(true);
//                Self::new(ProtocolVersion::new(1,4,0), c)
//            })
//            .map_err(move |e| IgniteError::new_with_source(format!("Can not connect to the host {}", addr), Box::new(e)));

        // conn_future
        
        AsyncDataChannel{ver, conn}

//        let ver = negotiate_protocol_version(&mut conn, cfg)?;

//        Ok(Self::new(ver, conn))
//        unimplemented!();
    }
}