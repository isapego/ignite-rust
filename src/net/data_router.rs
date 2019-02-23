use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::rc::Rc;
use std::sync::Mutex;

use crate::ignite_configuration::IgniteConfiguration;
use crate::ignite_error::{IgniteError, IgniteResult, LogResult, ReplaceResult, RewrapResult};
use crate::net::end_point::ResolvedEndPoint;
use crate::net::utils;
use crate::net::EndPoint;
use crate::protocol::message::{HandshakeReq, HandshakeRsp};
use crate::protocol::{Pack, Readable, Unpack, Writable};
use crate::protocol_version::ProtocolVersion;

/// Component which is responsible for establishing and
/// maintaining reliable connection link to the Ignite cluster.
///
/// It also responsible for choosing which connection to use for
/// a certain request.
#[derive(Debug)]
pub struct DataRouter {
    cfg: Rc<IgniteConfiguration>,
    conn: Mutex<Option<TcpStream>>,
}

impl DataRouter {
    /// Make new instance
    pub fn new(cfg: Rc<IgniteConfiguration>) -> Self {
        Self {
            cfg,
            conn: Mutex::new(None),
        }
    }

    /// Try establish connection with the address
    fn try_connect(addr: &SocketAddr) -> IgniteResult<TcpStream> {
        let stream = TcpStream::connect(addr)
            .rewrap_on_error(format!("Failed to connect to remote host {}", addr))?;

        stream.set_nonblocking(true).rewrap_on_error(format!(
            "Failed to set connection to non-bloaking mode for host {}",
            addr
        ))?;

        stream.set_nodelay(true).log_w_on_error(format!(
            "Failed to set connection to no-delay mode for host {}",
            addr
        ));

        Ok(stream)
    }

    /// Try perform handshake with the specified version
    fn handshake(&mut self, ver: ProtocolVersion) -> IgniteResult<()> {
        let req = HandshakeReq::new(ver, self.cfg.get_user(), self.cfg.get_password());
        let req_data = req.pack();

        let lock = self
            .conn
            .get_mut()
            .replace_on_error("Connection is probably poisoned")?;

        let conn = lock
            .as_mut()
            .expect("Should never be called on closed connection");

        conn.write_all(&req_data)
            .rewrap_on_error("Can not send handshake request")?;

        let rsp_data =
            Self::receive_raw_rsp(conn).rewrap_on_error("Can not receive handshake response")?;

        let rsp = HandshakeRsp::unpack(&rsp_data);

        Ok(())
    }

    /// Receive response in a raw byte array form
    fn receive_raw_rsp(conn: &mut TcpStream) -> IgniteResult<Box<[u8]>> {
        use crate::protocol::utils;

        let mut len_buf = [0u8; 4];

        conn.read_exact(&mut len_buf)
            .rewrap_on_error("Error while reading response length")?;

        let len = utils::deserialize_i32(&len_buf);

        let mut buf = vec![0u8; len as usize].into_boxed_slice();

        conn.read_exact(&mut buf)
            .rewrap_on_error("Error while reading response payload")?;

        Ok(buf)
    }

    /// Try establish initial connection with Ignite cluster
    pub fn initial_connect(&mut self) -> IgniteResult<()> {
        let mut end_points = self.cfg.get_endpoints().to_owned();

        &mut end_points[..].shuffle(&mut thread_rng());

        let resolved: Vec<ResolvedEndPoint> = end_points
            .iter()
            .filter_map(|x| {
                x.resolve()
                    .log_w_on_error(format!("Can not resolve host {}", x.host()))
            })
            .collect();

        for end_point in resolved {
            for addr in end_point {
                let res = Self::try_connect(&addr)
                    .log_w_on_error(format!("Can not connect to the host {}", addr));

                let stream = match res {
                    Some(s) => s,
                    None => continue,
                };

                // TODO: Implement handshake here

                // We do not care if the inner value is poisoned, as we are going to reassign it
                // without reading.
                let lock = self.conn.get_mut().unwrap_or_else(|e| e.into_inner());

                *lock = Some(stream);

                return Ok(());
            }
        }

        Err(IgniteError::new("Can not connect to any host"))
    }
}
