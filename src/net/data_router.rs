use log::Level;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::rc::Rc;

use ignite_configuration::IgniteConfiguration;
use ignite_error::{HandleResult, IgniteResult};
use net::end_point::ResolvedEndPoint;
use net::utils;

/// Component which is responsible for establishing and
/// maintaining reliable connection link to the Ignite cluster.
///
/// It aslo responsible for choosing which connection to use for
/// a certain request.
#[derive(Debug)]
pub struct DataRouter {
    cfg: Rc<IgniteConfiguration>,
    conns: HashMap<SocketAddr, TcpStream>,
}

impl DataRouter {
    /// Make new instance
    pub fn new(cfg: Rc<IgniteConfiguration>) -> DataRouter {
        DataRouter {
            cfg: cfg,
            conns: HashMap::new(),
        }
    }

    fn try_connect(addr: &SocketAddr) -> IgniteResult<TcpStream> {
        let stream = TcpStream::connect(addr)
            .rewrap_on_error(format!("Failed to connect to remote host {}", addr))?;

        stream.set_nonblocking(true).rewrap_on_error(format!(
            "Failed to set connection to non-bloaking mode for host {}",
            addr
        ))?;

        stream.set_nodelay(true).log_on_error(
            Level::Warn,
            format!(
                "Failed to set connection to no-delay mode for host {}",
                addr
            ),
        );

        Ok(stream)
    }

    /// Try establish initial connection with Ignite cluster
    pub fn initial_connect(&mut self) -> IgniteResult<()> {
        let mut end_points = utils::parse_endpoints(self.cfg.get_endpoints())?;

        &mut end_points[..].shuffle(&mut thread_rng());

        let resolved: Vec<ResolvedEndPoint> = end_points
            .iter()
            .filter_map(|x| {
                x.resolve()
                    .log_on_error(Level::Warn, format!("Can not resolve host {}", x.host()))
            })
            .collect();

        for end_point in resolved {
            for addr in end_point {
                let res = Self::try_connect(&addr)
                    .log_on_error(Level::Warn, format!("Can not connect to the host {}", addr));

                let stream = match res {
                    Some(s) => s,
                    None => continue,
                };

                // TODO: Implement handshake here

                self.conns.insert(addr, stream);

                return Ok(());
            }
        }

        Err("Can not connect to any host".into())
    }
}
