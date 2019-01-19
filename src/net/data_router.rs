use rand::seq::SliceRandom;
use rand::thread_rng;
use std::net::{SocketAddr, TcpStream};
use std::rc::Rc;

use crate::ignite_configuration::IgniteConfiguration;
use crate::ignite_error::{HandleResult, IgniteResult};
use crate::net::end_point::ResolvedEndPoint;
use crate::net::utils;
use crate::protocol_version::ProtocolVersion;

/// Component which is responsible for establishing and
/// maintaining reliable connection link to the Ignite cluster.
///
/// It aslo responsible for choosing which connection to use for
/// a certain request.
#[derive(Debug)]
pub struct DataRouter {
    cfg: Rc<IgniteConfiguration>,
    conn: Option<TcpStream>,
}

impl DataRouter {
    /// Make new instance
    pub fn new(cfg: Rc<IgniteConfiguration>) -> DataRouter {
        DataRouter {
            cfg: cfg,
            conn: None,
        }
    }

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

    // Try perform handshake with the specified version
    fn handshake(&mut self, _ver: &ProtocolVersion) -> IgniteResult<()> {
        assert!(
            self.conn.is_some(),
            "Should never be called without opening connection"
        );

        Ok(())
    }

    /// Try establish initial connection with Ignite cluster
    pub fn initial_connect(&mut self) -> IgniteResult<()> {
        let mut end_points = utils::parse_endpoints(self.cfg.get_endpoints())?;

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

                self.conn = Some(stream);

                return Ok(());
            }
        }

        Err("Can not connect to any host".into())
    }
}
