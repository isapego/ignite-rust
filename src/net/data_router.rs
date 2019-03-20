use rand::seq::SliceRandom;
use rand::thread_rng;
use std::borrow::BorrowMut;
use std::cell::Cell;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::rc::Rc;
use std::sync::Mutex;

use crate::ignite_configuration::IgniteConfiguration;
use crate::ignite_error::{IgniteError, IgniteResult, LogResult, ReplaceResult, ChainResult};
use crate::net::end_point::ResolvedEndPoint;
use crate::net::utils;
use crate::net::EndPoint;
use crate::protocol::message::{HandshakeReq, HandshakeRsp, Response};
use crate::protocol::{Pack, Readable, Unpack, Writable};
use crate::protocol_version::{ProtocolVersion, VERSION_1_3_0};

/// Default version
const VERSION_DEFAULT: ProtocolVersion = VERSION_1_3_0;

/// Versions supported by the client
const SUPPORTED_VERSIONS: [ProtocolVersion; 1] = [VERSION_1_3_0];

/// Component which is responsible for establishing and maintaining reliable
/// connection link to the Ignite cluster.
///
/// It also responsible for choosing which connection to use for a certain
/// request.
#[derive(Debug)]
pub struct DataRouter {
    cfg: Rc<IgniteConfiguration>,
    conn: Mutex<Option<TcpStream>>,
    ver: Cell<ProtocolVersion>,
}

impl DataRouter {
    /// Make new instance
    pub fn new(cfg: Rc<IgniteConfiguration>) -> Self {
        Self {
            cfg,
            conn: Mutex::new(None),
            ver: Cell::new(VERSION_DEFAULT),
        }
    }

    /// Try establish connection with the address
    fn try_connect(addr: &SocketAddr) -> IgniteResult<TcpStream> {
        let stream = TcpStream::connect(addr)
            .chain_error(format!("Failed to connect to remote host {}", addr))?;

        stream.set_nonblocking(true).chain_error(format!(
            "Failed to set connection to non-bloaking mode for host {}",
            addr
        ))?;

        stream.set_nodelay(true).log_error_w(format!(
            "Failed to set connection to no-delay mode for host {}",
            addr
        ));

        Ok(stream)
    }

    /// Receive response in a raw byte array form
    fn receive_raw_rsp(conn: &mut TcpStream) -> IgniteResult<Box<[u8]>> {
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

    /// Send request and receive a response as a byte buffers
    fn send_request_raw(&self, req: &[u8]) -> IgniteResult<Box<[u8]>> {
        let mut lock = self
            .conn
            .lock()
            .replace_error("Connection is probably poisoned")?;

        let mut conn = lock
            .as_mut()
            .expect("Should never be called on closed connection");

        conn.write_all(&req)
            .chain_error("Can not send request")?;

        Self::receive_raw_rsp(&mut conn).chain_error("Can not receive response")
    }

    /// Send a request and get a response.
    pub fn send_request<Req, Resp>(&self, req: Req) -> IgniteResult<Resp::Item>
    where
        Req: Pack,
        Resp: Unpack,
    {
        let req_data = req.pack();

        let rsp_data = self.send_request_raw(&req_data)?;

        Ok(Resp::unpack(&rsp_data))
    }

    /// Try perform handshake with the specified version
    fn handshake(
        conn: &mut TcpStream,
        cfg: &IgniteConfiguration,
        ver: ProtocolVersion,
    ) -> IgniteResult<HandshakeRsp> {
        let req = HandshakeReq::new(ver, cfg.get_user(), cfg.get_password());
        let req_data = req.pack();

        conn.write_all(&req_data)
            .chain_error("Can not send handshake request")?;

        let rsp_data =
            Self::receive_raw_rsp(conn).chain_error("Can not receive handshake response")?;

        Ok(HandshakeRsp::unpack(&rsp_data))
    }

    /// Try to negotiate connection parameters on a freshly open connection
    fn negotiate_connection(
        conn: &mut TcpStream,
        cfg: &IgniteConfiguration,
    ) -> Option<ProtocolVersion> {
        for ver in SUPPORTED_VERSIONS.iter() {
            let res = Self::handshake(conn, cfg, ver.clone())
                .log_error_w(format!("Handshake failed with version {:?}", ver));

            let resp = match res {
                Some(r) => r,
                None => continue,
            };

            match resp {
                Response::Accept(_) => return Some(ver.clone()),
                Response::Reject(rej) => warn!("Handshake failed with error: {}", rej.get_error()),
            }
        }

        None
    }

    /// Try establish connection with Ignite cluster
    pub fn establish_connection(&self) -> IgniteResult<()> {
        let mut end_points = self.cfg.get_endpoints().to_owned();

        &mut end_points[..].shuffle(&mut thread_rng());

        let resolved = end_points.iter().filter_map(|x| {
            x.resolve()
                .log_error_w(format!("Can not resolve host {}", x.host()))
        });

        for end_point in resolved {
            for addr in end_point {
                let res = Self::try_connect(&addr)
                    .log_error_w(format!("Can not connect to the host {}", addr));

                let mut stream = match res {
                    Some(s) => s,
                    None => continue,
                };

                let maybe_ver = Self::negotiate_connection(&mut stream, &self.cfg);

                let ver = match maybe_ver {
                    Some(v) => v,
                    None => continue,
                };

                self.ver.set(ver);

                // We do not care if the inner value is poisoned, as we are going to reassign it
                // without reading.
                let mut lock = self.conn.lock().unwrap_or_else(|e| e.into_inner());

                *lock = Some(stream);

                return Ok(());
            }
        }

        Err(IgniteError::new("Can not connect to any host"))
    }
}
