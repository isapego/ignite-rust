use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::rc::Rc;

use ignite_configuration::IgniteConfiguration;
use ignite_error::{IgniteResult, WrapOnError};
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
            .wrap_on_error(format!("Failed to connect to remote host {}", addr))?;

        // TODO: replace error with warning here
        stream.set_nodelay(true).wrap_on_error(format!(
            "Failed to set connection to no-delay mode for host {}",
            addr
        ))?;
        stream.set_nonblocking(true).wrap_on_error(format!(
            "Failed to set connection to non-bloaking mode for host {}",
            addr
        ))?;

        Ok(stream)
    }

    /// Try establish initial connection with Ignite cluster
    pub fn initial_connect(&mut self) -> IgniteResult<()> {
        let mut end_points = utils::parse_endpoints(self.cfg.get_endpoints())?;

        &mut end_points[..].shuffle(&mut thread_rng());

        // TODO: Add logging here
        let resolved: Vec<ResolvedEndPoint> =
            end_points.iter().filter_map(|x| x.resolve().ok()).collect();

        for end_point in resolved {
            for addr in end_point {
                // TODO: Add logging here
                let stream = Self::try_connect(&addr)?;
                self.conns.insert(addr, stream);

                return Ok(());
            }
        }

        Err("Can not connect to any host".into())
    }
}
